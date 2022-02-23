# todo

### prereqs
0. Draw out what the graph will look like
0. Scrape the sitemap for forecast IDs (periodically - thread for web server, cmd for cli?)
0. Cache in file system or DB or what? Track in git and load at compile time?

### app runtime
1. hit MSW API (only fields we need)
2. cache the results (OR NOT?) (invalidate every 3hrs) db for server ? .cache directory for cli?
3. models representing the forecast data. potentially parse the forecast data at
   the time of fetching into more efficient format for getting into models?

### app config
4. secrets mgmt for api key


# Dev Plan
1. Maybe for now require path `curl thesurf.in/ormond-beach`
2. Then allow disambiguation `thesurf.in/ormond-beach-ca` or `thesurf.in/ormond-beach-fl`
3. Then use user agent location plus lat/long scraped from MSW to allow `curl
   thesurf.in` to find the closest one.


##### get the location data for each spot?
- Go to https://magicseaweed.com/Ruby-Beach-Surf-Guide/5872/
- Then find
      data-guide='[{"spot":{"id":308,"name":"La Push","offset":-28800,"lat":47.9029,"lon":-124.634,
  in the HTML
- So yeah, we'll need to crawl the sitemap and then cache this, possibly save it in git and
  rarely update it?
