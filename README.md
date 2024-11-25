# Raytracer

- Based on: [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)
- Expanded on in the [UU Advanced Graphics](https://ics-websites.science.uu.nl/docs/vakken/magr/2024-2025) course

## Usage
```shell
cargo run --release -- scenes/weekend-final.json # Or another json scene
```
This will load a scene from a json file and render it to the `output` folder.
```sh
cargo run --release -- 
```
This will generate a new scene based on the code in `main.rs` (e.g. `let (world, filename) = scenes::simple_fuzzy_metal();`)  
It will write the scene as json to the `scenes` folder. (and also render to the `output` folder)

## Features / implementation details
- [x] All features from [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)
- [ ] Triangle geometry and intersection based on [this tutorial](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/geometry-of-a-triangle.html)

![Final image](final_image.png)