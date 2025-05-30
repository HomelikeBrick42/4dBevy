import include.camera;
import include.ray;

[vk::binding(0, 0)]
ConstantBuffer<Camera, Std430DataLayout> camera;

struct VertexOutput
{
    float4 clip_position : SV_Position;
    float2 uv;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID)
{
    var out : VertexOutput;
    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);

    out.uv = float2(x, y) * 2.0 - 1.0;
    out.clip_position = float4(out.uv, 0.0, 1.0);

    return out;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;

    var ray : Ray;
    ray.origin = camera.transform.transform(float4(0.0, 0.0, 0.0, 0.0));

    let rotation = camera.transform.rotor_part();
    let forward = rotation.rotate(float4(1.0, 0.0, 0.0, 0.0));
    let right = rotation.rotate(float4(0.0, 0.0, 1.0, 0.0));
    let up = rotation.rotate(float4(0.0, 1.0, 0.0, 0.0));
    ray.direction = normalize(forward + right * in.uv.x * camera.aspect + up * in.uv.y);

    out.color = float4(sky_color(ray), 1.0);
    return out;
}

float3 sky_color(Ray ray)
{
    let up_sky_color = float3(0.4, 0.5, 0.8);
    let down_sky_color = float3(0.1, 0.2, 0.4);
    return lerp(down_sky_color, up_sky_color, ray.direction.y * 0.5 + 0.5);
}
