# pixel-to-svg

**pixel-to-svg** is a tool for converting bitmap images to scalable vectors,
with a focus on pixel-perfection and tiny results.

developed as a modern alternative to florian berger's python2 program,
[pixel2svg], that utilises polygons to draw connected shapes, producing a
smaller SVG while maintaining one-to-one pixel accuracy.

## supported formats

the CLI takes the following image formats as input:

- [BMP] (`.bmp`, `.dib`)
- [ICO] (`.ico`)
- [PNG] (`.png`)
- [PNM] (`.pbm`, `.pgm`, `.ppm`, `.pnm`)
- [TGA] (`.tga`, `.icb`, `.vda`, `.vst`)

if you wish to convert from other file types, you can use the [magick] CLI to
convert to any of these. i wouldn't recommend using this tool with
lossily-compressed files (even after conversion) as they may contain
[compression artefacts] that could complicate the resulting SVG.

## supported methods

todo!

## related projects

- florian berger's [pixel2svg]
- valentin françois' [pixels2svg] (python3 rewrite of florian's)
- fluffy bucket snake's [pixel2svg-rs] (rust rewrite of florian's)
- peter selinger's [potrace] (not pixel-perfect, doesn't support colour)

[bmp]: https://en.wikipedia.org/wiki/BMP_file_format
[ico]: https://en.wikipedia.org/wiki/ICO_(file_format)
[png]: https://en.wikipedia.org/wiki/PNG
[pnm]: https://en.wikipedia.org/wiki/Netpbm
[tga]: https://en.wikipedia.org/wiki/Truevision_TGA
[magick]: https://github.com/ImageMagick/ImageMagick
[compression artefacts]: https://en.wikipedia.org/wiki/Compression_artifact
[pixel2svg]: https://florian-berger.de/en/software/pixel2svg
[pixel2svg-rs]: https://github.com/FluffyBucketSnake/pixel2svg-rs
[pixels2svg]: https://github.com/ValentinFrancois/pixels2svg
[potrace]: https://potrace.sourceforge.net/
