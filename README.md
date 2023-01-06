# imgprssr - a lean, mean, image resizing service.

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/leemartin77/imgprssr/test.yml?label=Test%20and%20Build) ![GitHub release (latest by date)](https://img.shields.io/github/v/release/leemartin77/imgprssr)

## What?

This is a very trim application - you give it a source (either a root directory, or a root web address), and it will allow images from this source to be resized on the fly.

It's written in rust, and relies heavily on the [hyper](https://crates.io/crates/hyper) and [image](https://crates.io/crates/image) crates, and in a lot of ways can be considered a simple wrapper around them.

## Why?

I wanted a container I could deploy to my cluster, which could serve and dynamically resize a directory of images, and was a bit shocked I couldn't immediately find something. Not for anything important, just a website for ranking mince pies. I thought I can't be the only one with this requirement - so I'm putting this up for others to use.

As to the _deeper_ why - it was because I wanted to send as little data as necessary over the wire, without having to statically bake sized images. I was looking at the directory of images as a datasource, rather than as images in and of themselves, due to the nature of the project.

## How?

### Configuring

The following environment variables are available:

- `IMGPRSSR_ADDRESS`: defaults to `0.0.0.0`
  - recommend not changing this, as it binds to the container
- `IMGPRSSR_PORT`: defaults to `3000`
  - the port that will be listened to
- `IMGPRSSR_IMAGE_SOURCE`: defaults to `./images`
  - the root directory to source images. Can be either a folder or a `http://`/`https://` web address
- `IMGPRSSR_DEFAULT_FILTER`: defaults to `nearest`
  - can be one of `nearest`, `gaussian`, `catmullrom`, `lanczos3`, `triangle`

### Running

Volume-mounted Image Source:

```bash
podman run -d -v ./images:/images -p 3000:3000 ghcr.io/leemartin77/imgprssr:latest
```

HTTPS Image Source:

```bash
podman run -d -e IMGPRSSR_IMAGE_SOURCE=https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images -p 3000:3000 ghcr.io/leemartin77/imgprssr:latest
```

There is also a binary of the application included with the release, however I give no guaruntees on this working, and _strongly_ recommend using the container.

### Requesting

Once the service is running, you can make a request with the following query parameters:

- `width`: the width of the image you want in pixels
- `height`: the height of the image you want in pixels
- `filter`: the filtering you want to use for resizing
  - one of `nearest`, `gaussian`, `catmullrom`, `lanczos3`, `triangle`

Note: It's worth playing around with the `filter` parameter based on the content of the image.

#### Examples:

```
http://localhost:3000/test_card_sml.png?width=300
```

Returns: The image at `/test_card_sml.png`, relative to the source, resized to 300px wide with aspect ratio preserved

```
http://localhost:3000/somedirectory/myimage.png?height=300
```

Returns: The image at `/somedirectory/myimage.png`, relative to the source, resized to 300px high with aspect ratio preserved

```
http://localhost:3000/test_card_sml.png?width=300&height=100
```

Returns: The image at `/test_card_sml.png`, relative to the source, resized so the longest edge is entirely in frame, and the short edge has the excess cropped. So:

![Uncropped 300 wide](docs/test_card_sml_300_wide_example.png)

Displays as:

![Cropped 300 wide 100 high](docs/test_card_sml_300_100_crop_example.png)

## Contributing?

I'm not actively looking for contributions on this since it's such a thin project, that said if there is something you need and you're willing to add it, please fork then raise a PR back to this repository - can't promise I'll merge it but I'll always be interested!

If you use this project, I'd _love_ to hear where you used it! You can reach me a [hello@leejohnmartin.co.uk](mailto:hello@leejohnmartin.co.uk).

## Other Notes

Test card image lifted from [SVGtestcard](https://github.com/edent/SVGtestcard)
