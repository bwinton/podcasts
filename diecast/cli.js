#!/usr/bin/env node

var ffmpeg = require('fluent-ffmpeg-extended');
var fs = require('fs');
var handlebars = require('handlebars');
var JSDOM = require('jsdom').JSDOM;
var jquery = require('jquery');
var minimist = require('minimist');
var parseString = require('xml2js').parseString;
var request = require('request');
var url = require('url');

var xmlStart = '<?xml version="1.0" encoding="UTF-8"?>\n' +
'<rss xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd"\n' +
'     xmlns:dc="http://purl.org/dc/elements/1.1/"\n' +
'     xmlns:atom="http://www.w3.org/2005/Atom"\n' +
'     version="2.0">\n' +
'  <channel>\n' +
'    <title>The Diecast</title>\n' +
'    <link>http://www.shamusyoung.com/twentysidedtale/?cat=287</link>\n' +
'    <atom:link href="http://bwinton.github.io/podcasts/diecast.xml" rel="self" type="application/rss+xml"/>\n' +
'    <language>en-US</language>\n' +
'    <itunes:author>Shamus, Chris, Rutskan, and Josh</itunes:author>\n' +
'    <itunes:subtitle>Like Spoiler Warning, but with less video.</itunes:subtitle>\n' +
'    <itunes:summary>Spoiler Warning, but audio-only</itunes:summary>\n' +
'    <description>Spoiler Warning, but audio-only</description>\n' +
'    <itunes:explicit>no</itunes:explicit>\n' +
'    <itunes:keywords>games,spoilerwarning,shamus,chris,rutskarn,josh</itunes:keywords>\n' +
'    <itunes:owner>\n' +
'      <itunes:name>Shamus Young</itunes:name>\n' +
'      <itunes:email>diecast@shamusyoung.com</itunes:email>\n' +
'    </itunes:owner>\n' +
'    <itunes:category text="Games &amp; Hobbies">\n' +
'      <itunes:category text="Video Games"/>\n' +
'    </itunes:category>\n' +
'    <itunes:image href="http://www.shamusyoung.com/twentysidedtale/images/splash_diecast2.jpg"/>\n';

var xmlEnd = '  </channel>\n' +
'</rss>\n';

var template = handlebars.compile('    <item>\n' +
'      <title>{{title}}</title>\n' +
'      <dc:creator>Shamus Young</dc:creator>\n' +
'      <pubDate>{{date}}</pubDate>\n' +
'      <link>{{link}}</link>\n' +
'      <guid isPermaLink="false">{{link}}</guid>\n' +
'      <description><![CDATA[{{{descHtml}}}]]></description>\n' +
'      <itunes:author>The Diecast</itunes:author>\n' +
'      <itunes:summary>{{descText}}</itunes:summary>\n' +
'      <itunes:explicit>no</itunes:explicit>\n' +
'      <itunes:duration>{{duration}}</itunes:duration>\n' +
'      <itunes:image href="http://www.shamusyoung.com/twentysidedtale/images/splash_diecast2.jpg"/>\n' +
'      <enclosure url="{{audio}}" length="{{length}}" type="audio/mpeg"/>\n' +
'    </item>\n');

function makeDate(date) {
  var rv = new Date(date + " 05:37:00 +0000");
  rv = rv.toGMTString().replace("GMT", "+0000");
  return rv;
}

function outputAll(headers) {
  out = xmlStart;
  headers.map(function (item, index) {
    console.error(index + ': Printing ' + item.title);
    if (item.audio) {
      var tmpl = template(item);
      out += tmpl;
    }
  });
  out += xmlEnd;
  console.log(out);
}

function addCachedData(cachedItem, header) {
  header.cached = true;
  header.descHtml = cachedItem['description'][0]
    .replace(']]>', ']] >')
    .replace(/<\/?script[^>]*>/g, '');
  header.descText = cachedItem['itunes:summary'][0];
  header.audio = cachedItem.enclosure[0].$.url;
  header.length = cachedItem.enclosure[0].$.length;
  header.duration = cachedItem['itunes:duration'][0];
}

function checkFinished(count, headers) {
  count++;
  if (count == headers.length) {
    outputAll(headers);
    console.error("Done!");
    process.exit();
  }
  return count;
}

function processUrl(baseUrl, cachedItems) {
  cachedItems = cachedItems || [];
  request.get(baseUrl, function (error, response, body) {
    if (error || response.statusCode !== 200) {
      console.error('Error:', error, response && response.statusCode);
    }
    var $ = jquery(new JSDOM(body).window);
    var items = $('.entry');
    console.error(items.length + ' entries found.');
    var headers = [];
    items.map(function (index, item) {
      item = $(item);
      var header = {};
      header.link = url.resolve(baseUrl, item.find('a[rel=bookmark]').attr('href'));
      var cachedItem = cachedItems.filter(function (cachedItem) {
        return cachedItem.link[0] === header.link;
      });
      console.error('Processing', index, header.link, cachedItem.length);
      header.title = item.find('.splash-title').text();
      // console.log();
      header.date = makeDate(item.find('.splash-avatar')[0].textContent.replace(/.*on /, ''));
      if (cachedItem.length) {
        addCachedData(cachedItem[0], header);
      }
      headers.push(header);
    });

    count = 0;
    headers.map(function (item, index) {
      if (item.cached) {
        console.error('Got cached ' + item.title);
        count = checkFinished(count, headers);
        return;
      }
      request.get(item.link, function (error, response, body) {
        console.error('Got ' + item.title, error);
        var $ = jquery(new JSDOM(body).window);
        var pDivs = $('.entry-text > p, .entry-text > ol, .entry-text > ul, .entry-text > blockquote')
                    .not('.entry-text > p.entry-tags');
        item.descHtml = "";
        item.descText = "";
        for (i = 0; i < pDivs.length; i++) {
          var div = $(pDivs[i]);
          var temp = div.find('a');
          temp.each(function (i, e) {
            $(e).attr('href', url.resolve(baseUrl, $(e).attr('href')));
          });
          var audio = div.find('audio > source[type="audio/mpeg"]');
          if (audio.length) {
            item.audio = url.resolve(item.link, audio.attr('src'));
            continue;
          }
          if (div.children().length && div.children()[0].tagName === "TABLE")
            continue;
          if (div[0].outerHTML.trim() !== "")
            item.descHtml += div[0].outerHTML.trim() + '\n';
          if (div.text().trim() !== "")
            item.descText += div.text().trim() + '\n\n';
        }
        if (!item.audio) {
          count = checkFinished(count, headers);
          return
        }
        request.head(item.audio, function (error, response, body) {
          console.error('Got audio for ' + item.audio, error);
          item.length = response.headers['content-length'];
          var proc = new ffmpeg.Metadata(item.audio, function (metadata, err) {
            item.duration = metadata.durationraw.replace(/\.\d\d$/, "")
            count = checkFinished(count, headers);
          });
        });
      });
    });
  });
}


function main(argv) {
  if (!argv._.length) {
    console.error('Please specify an input file.');
    return;
  }
  var file = argv._[0];
  var baseUrl = 'http://www.shamusyoung.com/twentysidedtale/?cat=287';

  fs.readFile(file, 'utf8', function (err, body) {
    if (err) {
      return console.log(err);
    }
    parseString(body, function (err, result) {
      var items = result.rss.channel[0].item;
      processUrl(baseUrl, items);
    });
  });
}
main(minimist(process.argv.slice(2)));



