## Implementation details

The feedback happens between two OpenGL textures in a ping-pong fashion. These
have a fixed square size, regardless of the size of the window or screen. It's
determined by the constant `FEEDBACK_TEXTURE_SIZE` in `src/main.rs`.

Each step of the feedback inverts colors, which produces a lot of the
interesting structure. This is why we only render every other step to the
screen; otherwise it would be far too blinky.

The shader programs (as GLSL source code strings) are baked into the azurescens
executable at compile time. This means the executable is self-contained and
does not rely on any external files. However, it also means you need to
recompile every time you change a shader. We plan to introduce a "dynamic
shaders" option to speed up the code/build/run cycle.


## Experimentation and improvement

The actual feedback function is implemented in `src/shaders/feedback.glsl`,
which is probably the most interesting file in the whole project. This is a
great place to start experimenting. There are a million different directions
you can go with this basic idea. Pull requests will be accepted, especially if
they add functionality without removing any. (We will need a mode-switching
interface at some point.)

Many ideas for improvement are available in the [issue tracker][issue]. See
also some old articles:
[1](http://wealoneonearth.blogspot.com/2007/09/more-fractal-video-feedback.html),
[2](http://wealoneonearth.blogspot.com/2007/09/more-screenshots.html),
[3](http://wealoneonearth.blogspot.com/2008/01/ezeiz-c_24.html),
[4](http://wealoneonearth.blogspot.com/2008/01/ezeiz-c.html), and many others
from that group blog.

Long ago, I made [a similar program][phosphene] in x86-16 assembly which fits
in a master boot record — 446 bytes.

[phosphene]: https://github.com/kmcallister/phosphene
[issue]: https://github.com/kmcallister/azurescens/issues
