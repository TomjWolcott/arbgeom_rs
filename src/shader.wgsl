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

// struct Geometry {
//     variant: u32,
//     data: u32,
//     a: u32, b: u32
// }

// @group(0) @binding(1) var<uniform> geometry_buffer: array<Geometry>;

@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4(pos.x, pos.y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // if ((4.29473 * pos.x + 9.18936 * pos.y) % 10.0 < 9.5) {
    //     return vec4(0.5, 0.5, 0.5, 1.0);
    // }

    let scale = min(info.width, info.height);
    let centered_pos = info.px_size * vec4(2.0 * pos.x - info.width, info.height - 2.0 * pos.y, 0.0, 0.0) / scale;
    let p = project_onto_curve(info.p + centered_pos.x * info.x + centered_pos.y * info.y);

    var i: f32 = 0.0;
    var point: Point = Point(
        p, 
        normalize(p - project_onto_curve(info.p - info.focal_length * info.z))
    );

    loop {
        if (i >= info.max_iterations) {
            break;
        }

        if ((point.pos.y < 0.0 && (point.ray.y < 0.0 || (pos.x + pos.y) % 2.0 < 1.0))) {
            break;
        }

        // if (pow(point.pos.x, 2.0) + pow(point.pos.y, 2.0) < pow(0.2, 2.0)) {
        //     break;
        // }

        let w2 = 0.0001;

        // if (p(point.pos.x - 0.5) + p(point.pos.y - 0.3) + p(point.pos.z - 0.5) + p(point.pos.w) <= 2.0) {
        //     break;
        // }

        // if (p(point.pos.x - 0.5) + p(point.pos.y - 0.3) + p(point.pos.z - 0.5) + p(point.pos.w - 20.0) <= 2.0) {
        //     break;
        // }

        // if (p(point.pos.x - 0.15) + p(point.pos.y - 0.15) + p(point.pos.z - 0.15) <= 1.0) {
        //     break;
        // }

        i += 1.0;
        point = follow_curve(point);
    }

    if (i >= info.max_iterations) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    } else if (i == 0.0) {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }
    let w = 100.0;
    let t: f32 = (
        round(0.5*cos(w * point.pos.x)+0.5) +
        round(0.5*cos(w * point.pos.y)+0.5) + 
        round(0.5*cos(w * point.pos.z)+0.5) +
        round(0.5*cos(w * point.pos.w)+0.5) + 
        0.0
    ) / 5.0;

    return vec4<f32>(
        t, t, (i / info.max_iterations),
        1.0
    );
}

fn p(x: f32) -> f32 {
    return select(select(100.0, 0.0, abs(x) < 0.2), 1.0, abs(x) < 0.1);
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
    return 6.0 * normalize(pos - vec4(0.0, 0.0, 0.0, 6.0)) + vec4(0.0, 0.0, 0.0, 6.0);
}

fn hyperplane_project_onto_curve(pos: vec4<f32>) -> vec4<f32> {
    return vec4(pos.x, pos.y, pos.z, 0.0);
}

fn hypersphere_project_onto_curve(pos: vec4<f32>) -> vec4<f32> {
    return normalize(pos);
}