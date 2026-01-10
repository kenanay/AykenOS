# Faz 2 - UI Scene and Render Prototype

Scene schema (ABDF):
- UiScene: name/id, width/height, bg_color, root_widget_ref
- UiWidget: type, x,y,w,h, fg/bg color, text_ref, layout hints, child indices
- Widget types (v0.2): Container, Label, Button (passive), Chart (dummy), Canvas placeholder

Render pipeline:
- ui.render <scene>: load scene -> build widget tree -> render
- Layout: basic absolute positioning; container may offset children
- Drawing: OpenGL (or stub) draws rectangles/text; if GL unavailable, log-only fallback

Sysdash demo (sample):
- 800x600 bg dark
- Header label "AykenOS System Dashboard"
- Left stack: CPU Usage label, Memory Usage label
- Right chart: dummy CPU history polyline

Testing:
- Stub renderer path exercised in unit tests
- If GL backend present: single window render once, then exit
