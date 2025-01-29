# 🧨 OpenJönsson

A modern re-implementation of the classic Swedish point-and-click adventure game *Jönssonligan: Jakten på Mjölner*.

https://github.com/user-attachments/assets/774d43a6-ef0f-4089-ac34-dc55ee3983a6

## Current Status  
⚠️ Early prototype - not yet playable. Most of the basic systems work, but the actual gameplay is currently limited to walking around.

## Getting Started

### Asset Extraction (using Docker)
OpenJönsson uses original game assets. You need to provide your own legitimate copy of *Jönssonligan: Jakten på Mjölner* to proceed.

Extract and process the assets using [cgex](https://github.com/kottz/cgex). The easiest way to do this is by using the provided docker container. After all of the image and audio assets have been extracted, the images are upscaled from the original 480x360 resolution to 1920x1440.

1. Create a `disc_contents` directory and a `resources` directory for the output.
2. Insert game CD or mount your .iso and copy everything on the CD into the `disc_contents` folder.
4. Run the docker container with the command below. This can take a while.

```bash
mkdir disc_contents
mkdir resources
docker run --rm \
  -v ./disc_contents:/input:ro \
  -v ./resources:/output \
  kottz/cgex:latest
```

Then clone the project if you haven't already and copy the `resources` folder to `static/resources` in the OpenJönsson repository.

### Running the Game
```bash
git clone https://github.com/kottz/openjonsson
cp -r resources openjonsson/static/
cd openjonsson
cargo run --release
```

## Legal
OpenJönsson is not affiliated with Korkeken AB or the original game creators. You must provide original game assets from a legally acquired copy.
