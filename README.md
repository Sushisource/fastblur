I wanted to see if I could blur screenshots for my lockscreen faster than 
ImageMagick. This is faster, but ultimately the slowness comes from reading/writing
to pngs, and the fact that i3lock only accepts pngs. In reality, using i3lock-color
is a better option than this, but it was a fun little exercise.

The actual blurring code is wholesale converted from http://blog.ivank.net/fastest-gaussian-blur.html
