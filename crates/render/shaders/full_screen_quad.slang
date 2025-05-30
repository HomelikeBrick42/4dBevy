import include.camera;
import include.hyper_sphere;
import include.ray;
import include.objects_info;

[vk::binding(0, 0)]
ConstantBuffer<Camera, Std430DataLayout> camera;

[vk::binding(0, 1)]
ConstantBuffer<ObjectsInfo, Std430DataLayout> objects_info;

[vk::binding(1, 1)]
StructuredBuffer<HyperSphere, Std430DataLayout> hyper_spheres;

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

    let hit = intersect_scene(ray);
    if (hit.hasValue && hit.value.distance >= camera.min_ray_distance && hit.value.distance <= camera.max_distance)
    {
        let hit = hit.value;

        let color_ = hit.material.color;
        let color = float3(color_.red, color_.green, color_.blue);

        var sun_ray : Ray;
        sun_ray.origin = hit.position;
        sun_ray.direction = SUN_DIRECTION;
        let sun_hit = intersect_scene(sun_ray);

        let ambient_light = 0.1;

        if (sun_hit.hasValue)
            out.color = float4(color * ambient_light, 1.0);
        else
            out.color = float4(color * max(ambient_light, dot(hit.normal, sun_ray.direction)), 1.0);
    }
    else
    {
        out.color = float4(sky_color(ray), 1.0);
    }

    return out;
}

static const float4 SUN_DIRECTION = normalize(float4(-0.3, 1.0, 0.3, 0.0));

float3 sky_color(Ray ray)
{
    if (dot(SUN_DIRECTION, ray.direction) > 0.99)
        return float3(1.0, 1.0, 1.0);

    let up_sky_color = float3(0.4, 0.5, 0.8);
    let down_sky_color = float3(0.1, 0.2, 0.4);
    return lerp(down_sky_color, up_sky_color, ray.direction.y * 0.5 + 0.5);
}

Optional<Hit> intersect_scene(Ray ray)
{
    var hit : Optional<Hit> = none;
    for (var i = 0u; i < objects_info.hyper_sphere_count; i++)
    {
        let maybe_hit = hyper_spheres[i].intersect(ray);
        if (maybe_hit.hasValue)
            if (!hit.hasValue || maybe_hit.value.distance < hit.value.distance)
                hit = maybe_hit;
    }
    return hit;
}
