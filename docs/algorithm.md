## The algorithm

Here's how it works, in a nutshell.

We interpret the screen as a region of the [complex plane] ℂ, for example

> (-1 to 1) + (-1 to 1)*i*

We pick a function *f*: ℂ → ℂ. Say we want to render a new frame of the
animation. For each pixel, we interpret its coordinate as a complex number *z*.
We then copy the color from the previous frame at the point *f*(*z*). We also
draw some stuff on top in order to seed the iteration with interesting
structure.

A simple choice for *f* is

> *f*(*z*) = *z*<sup>2</sup> + *c*

for some complex parameter *c*. This produces images very similar to [Julia set
fractals]. (The Julia set is the set of points for which repeated iteration of
*f* does not fly off to infinity.)

However, the animation also displays interesting non-equilibrium behavior, if
we vary the parameter *c* in-between frames. In azurescens this is accomplished
by moving the mouse. Skilled pilots can achieve some very interesting effects.

[complex plane]: https://en.wikipedia.org/wiki/Complex_plane
[Julia set fractals]: https://en.wikipedia.org/wiki/Julia_set
