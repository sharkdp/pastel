# pastel

*A command-line tool to generate, analyze, convert and manipulate colors.*

![pastel in action](doc/pastel.gif)

## Tutorial

### Getting help

`pastel` provides a number of commands like `saturate`, `mix` or `paint`. To see a complete list, you can simply run
``` bash
pastel
```
To get more information about a specific subcommand (say `mix`), you can call `pastel mix -h` or `pastel help mix`.

### Composition

Many `pastel` commands can be composed by piping the output of one command to another, for example:
``` bash
pastel random | pastel mix red | pastel lighten 0.2 | pastel format hex
```

### Specifying colors

Colors can be specified in many different formats:
```
lightslategray
'#778899'
778899
789
'rgb(119, 136, 153)'
'119,136,153'
'hsl(210, 14.3%, 53.3%)'
```

Colors can be passed as positional arguments, for example:
```
pastel lighten 0.2 orchid orange lawngreen
```
They can also be read from standard input. So this is equivalent:
```
printf "%s\n" orchid orange lawngreen | pastel lighten 0.2
```
You can also explicitly specify which colors you want to read from the input. For example, this mixes `red` (which is read from STDIN) with `blue` (which is passed on the command line):
```
pastel color red | pastel mix - blue
```

## Installation


## Development


## Resources

Interesting Wikipedia pages:

* [Color difference](https://en.wikipedia.org/wiki/Color_difference)
* [CIE 1931 color space](https://en.wikipedia.org/wiki/CIE_1931_color_space)
* [CIELAB color space](https://en.wikipedia.org/wiki/CIELAB_color_space)
* [Line of purples](https://en.wikipedia.org/wiki/Line_of_purples)
* [Impossible color](https://en.wikipedia.org/wiki/Impossible_color)
* [sRGB](https://en.wikipedia.org/wiki/SRGB)
* [Color theory](https://en.wikipedia.org/wiki/Color_theory)
* [Eigengrau](https://en.wikipedia.org/wiki/Eigengrau)

Color names:

* [XKCD Color Survey Results](https://blog.xkcd.com/2010/05/03/color-survey-results/)
* [Peachpuffs and Lemonchiffons - talk about named colors](https://www.youtube.com/watch?v=HmStJQzclHc)
* [List of CSS color keywords](https://www.w3.org/TR/SVG11/types.html#ColorKeywords)

Maximally distinct colors:

* [How to automatically generate N "distinct" colors?](https://stackoverflow.com/q/470690/704831)
* [Publication on two algorithms to generate (maximally) distinct colors](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.65.2790)

Other articles and videos:

* [Color Matching](https://www.youtube.com/watch?v=82ItpxqPP4I)
* [Introduction to color spaces](https://ciechanow.ski/color-spaces/)
