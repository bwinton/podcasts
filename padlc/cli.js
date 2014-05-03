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
'    <title>Penny Arcade: Downloadable Content (home-grown)</title>\n' +
'    <link>http://www.penny-arcade.com/dlc</link>\n' +
'    <atom:link href="http://bwinton.latte.ca/padlc.xml" rel="self" type="application/rss+xml"/>\n' +
'    <language>en-US</language>\n' +
'    <itunes:author>Penny Arcade</itunes:author>\n' +
'    <itunes:subtitle>Downloadable Content</itunes:subtitle>\n' +
'    <itunes:summary>After a three year hiatus, Penny Arcade’s DLC Podcast was brought back to life when over 5,000 fans supported our Summer Kickstarter Project. Part of our campaign promise was to allow those that couldn’t support us at the time a way to "pay what you want" two weeks after the episodes aired to Kickstarter backers.\n' +
'\n' +
'Now, if "what you want" is nothing, not a problem! There’s a link to the right of the Pay What You Want Button where you can download the episode for free.\n' +
'\n' +
'This is also our test for using the Tinypass service, so please drop us a note to let us know how that experience is.\n' +
'\n' +
'Thank you again for supporting Penny Arcade, and enjoy the second season to DLC!</itunes:summary>\n' +
'    <description>After a three year hiatus, Penny Arcade’s DLC Podcast was brought back to life when over 5,000 fans supported our Summer Kickstarter Project. Part of our campaign promise was to allow those that couldn’t support us at the time a way to "pay what you want" two weeks after the episodes aired to Kickstarter backers.\n' +
'\n' +
'Now, if "what you want" is nothing, not a problem! There’s a link to the right of the Pay What You Want Button where you can download the episode for free.\n' +
'\n' +
'This is also our test for using the Tinypass service, so please drop us a note to let us know how that experience is.\n' +
'\n' +
'Thank you again for supporting Penny Arcade, and enjoy the second season to DLC!</description>\n' +
'    <itunes:explicit>yes</itunes:explicit>\n' +
'    <itunes:keywords>games</itunes:keywords>\n' +
'    <itunes:owner>\n' +
'      <itunes:name>Penny Arcade</itunes:name>\n' +
'      <itunes:email>pr@penny-arcade.com</itunes:email>\n' +
'    </itunes:owner>\n' +
'    <itunes:category text="Games &amp; Hobbies">\n' +
'      <itunes:category text="Video Games"/>\n' +
'    </itunes:category>\n' +
'    <itunes:image href="http://hw1.pa-cdn.com/pa/assets/img/bg_dlc.jpg"/>\n';

var xmlEnd = '  </channel>\n' +
'</rss>\n';

var template = handlebars.compile('    <item>\n' +
'      <title>{{title}}</title>\n' +
'      <dc:creator>Penny Arcade</dc:creator>\n' +
'      <pubDate>{{date}}</pubDate>\n' +
'      <link>{{link}}</link>\n' +
'      <guid isPermaLink="false">{{link}}</guid>\n' +
'      <description><![CDATA[{{descHtml}}]]></description>\n' +
'      <itunes:author>Penny Arcade</itunes:author>\n' +
'      <itunes:summary>{{descText}}</itunes:summary>\n' +
'      <itunes:explicit>yes</itunes:explicit>\n' +
'      <itunes:duration>{{duration}}</itunes:duration>\n' +
'      <itunes:image href="http://hw1.pa-cdn.com/pa/assets/img/bg_dlc.jpg"/>\n' +
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

function parseItem(item) {
  var header = {};
  var title;
  if (item.find('h4').length) {
    title = item.find('h4').text();
    header.link = url.resolve(baseUrl, item.find('a.paDLCLink').attr('href'));
    header.audio = header.link;
    header.date = makeDate(title.replace(/Episode \d+ - ([^ ]*).*/, '$1'));
    header.title = title.replace(/(Episode \d+ - )[^ ]* (.*)/, '$1$2');
    // Hacky solutions to missing dates.
    if (header.title === 'Episode 20 - Untitled') {
      header.date = makeDate('10/22/2013');
    } else if (header.title === 'Episode 15 - Sanderfuge') {
      header.date = makeDate('09/16/2013');
    }
    header.descHtml = '';
    header.descText = '';
  } else if (item.find('a[title]').length) {
    title = item.find('a').text() + ": " + item.find('em').text();
    header.link = url.resolve(baseUrl, item.find('a').attr('href'));
    header.audio = header.link;
    header.date = makeDate(title.replace(/([^ ]*).*/, '$1'));
    header.title = title.replace(/[^ ]* (.*)/, '$1')
    var desc = item.contents().not('a, em').text().trim();
    header.descHtml = desc;
    header.descText = desc;
  } else {
    header = null;
    console.error('Could not parse ' + item.html());
  }
  return header;
}

var baseUrl = 'http://www.penny-arcade.com/dlc';
request.get(baseUrl, function (error, response, body) {
  if (!error && response.statusCode == 200) {
    var document = jsdom(body);
    var window = document.createWindow();
    var $ = jquery.create(window);
    var items = $('h3 + ul > li');
    console.error(items.length + ' entries found.');
    var headers = [];
    items.map(function (index, item) {
      headers.push(parseItem($(item)));
    });

    count = 0;
    //console.log(headers);

    headers.map(function (item) {
      //console.log(index);
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
    });
  }
});

