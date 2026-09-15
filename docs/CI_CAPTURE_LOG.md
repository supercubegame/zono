# CI capture log

```
== zono UI capture ==
date   : Tue Sep 15 16:05:58 UTC 2026
binary : ./target/release/zono
os     : Linux
size   : 10776856 bytes
seeded 3 tasks (1 done)
-- installing capture tools --
tool ok   : Xvfb
tool ok   : import
tool ok   : xdotool
tool ok   : tesseract
renderer: ICED_BACKEND=tiny-skia
-- starting Xvfb --
-- launching zono --
window found after ~2s : id=2097154
captured : 23806 bytes
identify : dist/zono-ui-linux-x86_64.png PNG 1024x760 1024x760+0+0 8-bit sRGB 23806B 0.000u 0:00.000
colours  : 461 unique colours
stats    : mean=64774.2 sd=5632.56
-- OCR of the captured window --
zono
v0.2.1 - todo list with GUI
| Add a new task...
| Add
Ship zono v0.2.1
Write the README
Wire up the release pipeline
1 of 3 done
-- app stdout/stderr --
== capture finished ==
```
