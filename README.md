# todo

### qol
1. drop the cli
1. do basic key mapping: e.g. /Folly, /Folly-Beach, /FollyBeach,
   /folly-beach-sc, /folly-sc should all get to the same spot ID.

### ui/rendering
1. figure out graph design
1. render using a general Display type
1. impls for ANSI plain text rendering
1. impls for HTML default rendering

### auto location
1. crawl msw for lat/long of each spot (see [below](#crawl-lat-long)).
1. use https://ipinfo.io/signup to get lat/long of request
1. haversine to get closest one to request
1. profit

### crawl lat long
1. go to https://magicseaweed.com/Ruby-Beach-Surf-Guide/5872/
1. then find
      data-guide='[{"spot":{"id":308,"name":"La Push","offset":-28800,"lat":47.9029,"lon":-124.634,
  in the HTML
1. so yeah, we'll need to crawl the sitemap and then cache this, possibly save it in git and
  rarely update it?

### deployment
1. choose server e.g. linode
1. auto GH action deployment
1. secrets mgmt for msw api key
