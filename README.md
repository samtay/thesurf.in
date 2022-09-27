# thesurf.in
A console oriented surf forecast. **Still under development.**

# usage
You can view the content in your browser, but it's intended for the terminal.

|Operation|Command|
|---|---|
|**Forecast**|`curl thesurf.in/<spot-name>`|
|**List available spots**|`curl thesurf.in/spots`|
|**Find spot by name**|`curl thesurf.in/spots?search_substring`|

### examples

```shell
# get forecast for fire island
curl thesurf.in/fire-island

# find MSW's annoying name for mavericks
curl thesurf.in/spots?mavericks

# get the forecast for mavericks
curl thesurf.in/mavericks-half-moon-bay

# or with the MSW ID found from searching
curl thesurf.in/162
```

# todo

### dev deps
1. [rustup](https://rustup.rs/)
2. [1pw cli](https://developer.1password.com/docs/cli/get-started#install)
3. [just](https://github.com/casey/just#installation)

### todo
1. finish rendering daily view
1. impls for HTML rendering
1. user agent parse to decide HTML / ANSI
1. allow passing through `&units=eu,us,uk`.

### misc
1. drop the cli? requires api key unless I decide to go all in on scraping
1. do basic key mapping: e.g. /Folly, /Folly-Beach, /FollyBeach,
   /folly-beach-sc, /folly-sc should all get to the same spot ID. If relying on
   a hashmap, could maybe rayon-parallelize a search for each key variation.
1. if relying on shuttle.rs, could pick up sqlx/postgres?
1. query only what we _want_ from MSW, e.g.
    http://magicseaweed.com/api/YOURAPIKEY/forecast/?spot_id=10&fields=timestamp,wind.*,condition.temperature
1. damn, no tides from MSW! can do Marea API for $5/month: https://api.marea.ooo/doc/v2#get-/tides or scrape MSW on the fly :/

### deployment
1. choose server e.g. linode
1. auto GH action deployment
1. secrets mgmt for msw api key
1. or just use shuttle.rs? `op inject` and send over the api key? it's not
   _that_ sensitive...

### stretch: auto location
1. crawl msw for lat/long of each spot (see [below](#crawl-lat-long)).
1. use https://ipinfo.io/signup to get lat/long of request
1. haversine to get closest one to request
1. profit

### stretch: crawl lat long
1. go to https://magicseaweed.com/Ruby-Beach-Surf-Guide/5872/
1. then find
      data-guide='[{"spot":{"id":308,"name":"La Push","offset":-28800,"lat":47.9029,"lon":-124.634,
  in the HTML
1. so yeah, we'll need to crawl the sitemap and then cache this, possibly save it in git and
  rarely update it?

# limitations

Most are accustomed to green == clean, blue == ok, red == choppy; but MSW
doesn't provide me with whether or not the wind is on/off/cross shore. So I'm
using the (probably not great) proxy of their faded stars. 0-1 faded => green,
2 => blue, 3-5 => red.
