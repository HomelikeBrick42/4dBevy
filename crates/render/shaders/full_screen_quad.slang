struct VertexOutput
{
    float4 clip_position : SV_Position;
    float2 uv;
}

struct FragmentOutput
{
    float4 color : SV_Target;
}

[vk::binding(0, 0)]
Texture2D texture;
[vk::binding(1, 0)]
SamplerState textureSampler;

[shader("vertex")]
VertexOutput vertex(uint vertex_index: SV_VertexID, uint instance_id: SV_InstanceID)
{
    var out : VertexOutput;
    let x = float((vertex_index >> 0) & 1);
    let y = float((vertex_index >> 1) & 1);

    if (instance_id == 1)
    {
        out.uv = float2(x, y);
        out.clip_position = float4(out.uv * 0.5 + 0.5, 0.0, 1.0);
    }
    else
    {
        out.uv = float2(x, y);
        out.clip_position = float4(out.uv * 2.0 - 1.0, 0.0, 1.0);
    }

    return out;
}

[shader("fragment")]
FragmentOutput fragment(VertexOutput in)
{
    var out : FragmentOutput;
    out.color = texture.Sample(textureSampler, in.uv);
    return out;
}
