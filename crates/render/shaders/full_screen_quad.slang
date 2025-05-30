import include.camera;

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
    out.color = float4(in.uv * float2(camera.aspect, 1.0), 0.0, 1.0);
    return out;
}
