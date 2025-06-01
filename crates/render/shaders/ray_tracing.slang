import include.camera;
import include.hyper_sphere;
import include.ray;
import include.objects_info;
import include.material;

[vk::binding(0, 0)]
WTexture2D main_texture;

[vk::binding(0, 1)]
ConstantBuffer<Camera, Std430DataLayout> camera;

[vk::binding(1, 1)]
ConstantBuffer<ObjectsInfo, Std430DataLayout> objects_info;

[vk::binding(0, 2)]
StructuredBuffer<Material, Std430DataLayout> materials;

[vk::binding(1, 2)]
StructuredBuffer<HyperSphere, Std430DataLayout> hyper_spheres;

[shader("compute")]
[numthreads(16, 16, 1)]
void ray_trace(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    main_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    let uv = (float2(global_index.xy) / float2(width, height)) * 2.0 - 1.0;

    var color = float3(0.0, 0.0, 0.0);

    var ray : Ray;
    ray.origin = camera.position;
    ray.direction = normalize(camera.forward + camera.right * uv.x * camera.aspect + camera.up * uv.y);

    let hit = intersect_scene(ray);
    if (hit.hasValue)
    {
        let hit = hit.value;

        let hit_color_ = materials[hit.material_id].base_color;
        let hit_color = float3(hit_color_.red, hit_color_.green, hit_color_.blue);

        var sun_ray : Ray;
        sun_ray.origin = hit.position;
        sun_ray.direction = SUN_DIRECTION;
        let sun_hit = intersect_scene(sun_ray);

        let ambient_light = 0.1;

        if (sun_hit.hasValue)
            color = hit_color * ambient_light;
        else
            color = hit_color * max(ambient_light, dot(hit.normal, sun_ray.direction));
    }
    else
    {
        color = sky_color(ray);
    }

    main_texture.Store(global_index.xy, float4(color, 1.0));
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
        let maybe_hit = hyper_spheres[i].intersect(ray, camera.min_distance, camera.max_distance);
        if (maybe_hit.hasValue)
            if (!hit.hasValue || maybe_hit.value.distance < hit.value.distance)
                hit = maybe_hit;
    }
    return hit;
}
