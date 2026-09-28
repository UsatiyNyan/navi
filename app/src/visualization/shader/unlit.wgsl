struct TransformFrame {
    rotation: vec4<f32>, // quat
    translation: vec3<f32>,
};

@group(0) @binding(0)
var<storage, read> s_transform_frames: array<TransformFrame>;

@group(1) @binding(0)
var<uniform> u_mvp: mat4x4<f32>;

// TODO: batch vertex input in SSBO
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct InstanceInput {
    @location(4) transation: vec3<f32>,
    @location(5) frame: u32,
    @location(6) rotation: vec4<f32>, // quat
    @location(7) scale: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

fn reconstruct_rotation(r: vec4<f32>) -> mat3x3<f32> {
    // expect normalized, length(r) == 1

    let x2 = r.x + r.x;
    let y2 = r.y + r.y;
    let z2 = r.z + r.z;
    let xx = r.x * x2;
    let yy = r.y * y2;
    let zz = r.z * z2;
    let xy = r.x * y2;
    let xz = r.x * z2;
    let yz = r.y * z2;
    let wx = r.w * x2;
    let wy = r.w * y2;
    let wz = r.w * z2;

    return mat3x3<f32>(
        vec3<f32>(1.0 - (yy + zz), xy + wz, xz - wy),
        vec3<f32>(xy - wz, 1.0 - (xx + zz), yz + wx),
        vec3<f32>(xz + wy, yz - wx, 1.0 - (xx + yy))
    );
}

fn reconstruct_transform(tf_rotation: vec4<f32>, tf_scale: vec3<f32>, tf_translation: vec3<f32>) -> mat4x4<f32> {
    let rotation = reconstruct_rotation(tf_rotation);
    let scale = mat3x3<f32>(
        tf_scale.x, 0.0, 0.0,
        0.0, tf_scale.y, 0.0,
        0.0, 0.0, tf_scale.z,
    );
    let rs = rotation * scale;
    return mat4x4<f32>(
        vec4<f32>(rs[0], 0.0),
        vec4<f32>(rs[1], 0.0),
        vec4<f32>(rs[2], 0.0),
        vec4<f32>(tf_translation, 1.0),
    );
}

@vertex
fn vs_main(
    vin: VertexInput,
    iin: InstanceInput,
) -> VertexOutput {
    let frame = s_transform_frames[iin.frame];
    let frame_tf = reconstruct_transform(frame.rotation, vec3<f32>(1.0, 1.0, 1.0), frame.translation);
    let instance_tf = reconstruct_transform(iin.rotation, iin.scale, iin.transation);

    var out: VertexOutput;
    out.clip_position = u_mvp * frame_tf * instance_tf * vec4<f32>(vin.position, 1.0);
    out.color = vin.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

