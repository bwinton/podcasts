var ffmpeg = require('fluent-ffmpeg-extended');
var handlebars = require('handlebars');
var jsdom = require('jsdom').jsdom;
var jquery = require('jquery');
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
'    <atom:link href="http://bwinton.latte.ca/diecast.xml" rel="self" type="application/rss+xml"/>\n' +
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
'      <description><![CDATA[{{descHtml}}]]></description>\n' +
'      <itunes:author>The Diecast</itunes:author>\n' +
'      <itunes:summary>{{descText}}</itunes:summary>\n' +
'      <itunes:explicit>no</itunes:explicit>\n' +
'      <itunes:duration>{{duration}}</itunes:duration>\n' +
'      <itunes:image href="http://www.shamusyoung.com/twentysidedtale/images/splash_diecast2.jpg"/>\n' +
'      <enclosure url="{{audio}}" length="{{length}}" type="audio/mpeg"/>\n' +
'    </item>\n');

// Episode 01 - 06/07/2013 Conversions
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

var baseUrl = 'http://www.shamusyoung.com/twentysidedtale/?cat=287';
request.get(baseUrl, function (error, response, body) {
  if (!error && response.statusCode == 200) {
    var document = jsdom(body);
    var window = document.createWindow();
    var $ = jquery.create(window);
    var items = $('.entry');
    console.error(items.length + ' entries found.');
    var headers = [];
    items.map(function (index, item) {
      console.error('Processing ' + index);
      item = $(item);
      var header = {};
      header.title = item.find('.splash-title').text();
      header.link = url.resolve(baseUrl, item.find('a[rel=bookmark]').attr('href'));
      header.date = makeDate(item.find('td:nth-child(5)')[0].textContent);
      headers.push(header);
    });

    count = 0;
    headers.map(function (item, index) {
      console.error('Getting ' + item.title);
      request.get(item.link, function (error, response, body) {
        console.error('Got ' + item.title, error);
        var document = jsdom(body);
        var window = document.createWindow();
        var $ = jquery.create(window);
	var pDivs = $('.entry-text > p, .entry-text > ol, .entry-text > ul')
                    .not('.entry-text > p.entry-tags');
        item.descHtml = "";
        item.descText = "";
	for (i = 0; i < pDivs.length; i++) {
          var div = $(pDivs[i]);
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
          count++;
          if (count == headers.length) {
            outputAll(headers);
            console.error("Done!");
            process.exit();
          } 
	}
	if (item.audio) {
          request.head(item.audio, function (error, response, body) {
            console.error('Got audio for ' + item.audio, error);
            item.length = response.headers['content-length'];
            var proc = new ffmpeg.Metadata(item.audio, function (metadata, err) {
              item.duration = metadata.durationraw.replace(/\.\d\d$/, "")
              count++;
              if (count == headers.length) {
                outputAll(headers);
                console.error("Done!");
                process.exit();
              } 
            });
          });
	}
      });
    });
  }
});

