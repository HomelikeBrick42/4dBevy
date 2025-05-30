import transform;

struct Camera
{
    Transform transform;
    float aspect;
    float min_ray_distance;
    float max_distance;
    uint max_bounces;
}
