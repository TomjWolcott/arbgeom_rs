// Pass in all of the needed info to render
struct Info {
    x: vec4<f32>,
    y: vec4<f32>,
    z: vec4<f32>,
    p: vec4<f32>,
    focal_length: f32,
    px_size: f32,
    width: f32,
    height: f32,
    delta: f32,
    max_iterations: f32,
    time: f32
}

@group(0) @binding(0) var<uniform> info: Info;

// Carries any variables that the projection function needs
struct ManifoldInfo {
    v1: vec4<f32>,
    v2: vec4<f32>,
    v3: vec4<f32>,
    r1: f32,
    r2: f32,
    r3: f32,
}

@group(0) @binding(1) var<uniform> manifold_info: ManifoldInfo;

// I'm not using the vertex shader for anything
// since raymarching is handling all of the rendering
@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4(pos.x, pos.y, 0.0, 1.0);
}

// All of the rendering happens here
@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let scale = min(info.width, info.height);
    let centered_pos = info.px_size * vec4(2.0 * pos.x - info.width, info.height - 2.0 * pos.y, 0.0, 0.0) / scale;
    // get the position of the center of the camera on the manifold
    let p = project_onto_curve(info.p + centered_pos.x * info.x + centered_pos.y * info.y);

    var i: f32 = 0.0;
    // Start the ray at the pixel pointing out into the world
    var point: Point = Point(
        p, 
        normalize(p - project_onto_curve(info.p - info.focal_length * info.z))
    );

    var add_color: vec4<f32>;

    loop {
        // If the ray has stepped forward too many times, then stop
        if (i >= info.max_iterations) {
            break;
        }

        // Adds in the red & blue horizontal planes
        if (abs(point.pos.y) < 0.1) {
            if (point.pos.y < 0.0) {
                add_color = vec4(0.3, 0.0, 0.0, 0.0);
            } else {
                add_color = vec4(0.0, 0.3, 0.3, 0.0);
            }

            break;
        }

        // Adds in the orange cube
        if (cube(point, vec4(0.0, 0.0, 0.0, 0.0)) <= 2.0) {
            add_color = vec4(0.3, 0.0, 0.0, 0.0);
            break;
        }

        i += 1.0;
        // Move forward and project back onto the manifold
        point = follow_curve(point);
    }

    // If the ray never found anything, color the pixel black
    if (i >= info.max_iterations) {
        return vec4(0.0, 0.0, 0.0, 1.0);

    // If the ray immediately found something, color the pixel white
    } else if (i == 0.0) {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }

    // Adds in the grid pattern on the objects
    let w = 20.0;
    let t: f32 = (
        round(0.5*cos(w * point.pos.x)+0.5) +
        round(0.5*cos(w * point.pos.y)+0.5) + 
        round(0.5*cos(w * point.pos.z)+0.5) +
        round(0.5*cos(w * point.pos.w)+0.5) + 
        0.0
    ) / 4.0;

    return (1.0 - i / info.max_iterations) * normalize(vec4<f32>(
        t, t, (i / info.max_iterations),
        1.0
    ) + add_color);
}

fn cube(point: Point, v: vec4<f32>) -> f32 {
    let c = project_onto_curve(v);
    return p(point.pos.x - c.x) + p(point.pos.y - c.y) + p(point.pos.z - c.z) + p(point.pos.w - c.w);
}

fn p(x: f32) -> f32 {
    return select(select(100.0, 0.0, abs(x) < 1.0), 1.0, abs(x) < 0.5);
}

struct Point {
    pos: vec4<f32>,
    ray: vec4<f32>
}

fn follow_curve_loop(point: Point, n: u32, direction: f32) -> Point {
    var new_point = Point(
        point.pos,
        direction * point.ray
    );

    for (var i: u32 = 0u; i < n; i++) {
        new_point = follow_curve(new_point);
    }

    return new_point;
}

fn follow_curve(point: Point) -> Point {
    let new_pos = project_onto_curve(point.pos + info.delta * point.ray);

    return Point(
        new_pos,
        normalize(new_pos - point.pos)
    );
}

// The real project fn will be injected here
fn project_onto_curve(pos: vec4<f32>) -> vec4<f32> {
}