# todo

1. Drop the cli? Neat but.. not really necessary. Keep a binary for updating
   spots json.
2. Do a basic key to mapping. E.g. /Folly, /Folly-Beach, /FollyBeach,
   /folly-beach-sc, /folly-sc should all get to the same spot ID.

### prereqs
1. Draw out what the graph will look like

### app config
1. secrets mgmt for api key

### questions for later
1. Periodically scrape the sitemap for forecast IDs?
2. External memory store e.g. redis or hashmap of spot IDs OK?
1. Cache the MSW responses? (invalidate every 3hrs)

# Dev Plan
1. Maybe for now require path `curl thesurf.in/ormond-beach`
1. Then allow disambiguation `thesurf.in/ormond-beach-ca` or `thesurf.in/ormond-beach-fl`
1. Then use user agent location plus lat/long scraped from MSW to allow `curl
   thesurf.in` to find the closest one.


##### get the location data for each spot?
- Go to https://magicseaweed.com/Ruby-Beach-Surf-Guide/5872/
- Then find
      data-guide='[{"spot":{"id":308,"name":"La Push","offset":-28800,"lat":47.9029,"lon":-124.634,
  in the HTML
- So yeah, we'll need to crawl the sitemap and then cache this, possibly save it in git and
  rarely update it?
