# [thesurf.in](https://samtay.github.io/thesurf.in)

A console oriented surf forecast.

**Notice**: When Surfline acquired MSW, they shut down their API. This site is
no longer functional. I've saved a [demo](https://samtay.github.io/thesurf.in/demo) of an old
forecast on GitHub pages.

![2023-01-19-190036_990x769_scrot](https://user-images.githubusercontent.com/7246591/213608811-13cbe3b5-9d1c-44eb-a700-73d4be15c3de.png)

## usage
You can view the content in your browser, but it's intended for the terminal.

|Operation|Command|
|---|---|
|**Demo Forecast**|`curl -L thesurf.in/demo`|
|~~**Spot Forecast**~~|~~`curl -L thesurf.in/<spot-name>`~~|
|**List available spots**|`curl -L thesurf.in/spots`|
|**Find spot by name**|`curl -L thesurf.in/spots?search_substring`|

### examples

```shell
# get forecast for fire island
curl https://thesurf.in/fire-island

# find MSW's annoying name for mavericks
curl https://thesurf.in/spots?mavericks

# get the forecast for mavericks
curl https://thesurf.in/mavericks-half-moon-bay

# or with the MSW ID found from searching
curl https://thesurf.in/162
```

### units
You can append the query param `?units={uk,us,eu}` to the forecast endpoint, where

- `us`: uses ft, mph, f
- `uk`: uses ft, mph, c
- `eu`: uses m, kph, c

These are passed directly to MSW
([docs](https://magicseaweed.com/docs/developers/59/units-of-measurement/9911/)).

## limitations

The MSW forecast data does not convey the relationship of the wind relative to
the shore (e.g. on/off/cross shore). The red/blue/green ratings in the interface
are instead a simple function of [MSW's faded
stars](https://magicseaweed.com/help/forecast-table/star-rating). This function
is not perfect, and could probably be improved.

## dev deps
1. [rust](https://rustup.rs/)
2. [1pw cli](https://developer.1password.com/docs/cli/get-started#install)
3. [just](https://github.com/casey/just#installation)
4. An MSW API key, which they are not currently offering to the general public.
