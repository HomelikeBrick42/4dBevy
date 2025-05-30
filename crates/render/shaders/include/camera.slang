import transform;

struct Camera
{
    Transform transform;
    float fov;
    float min_ray_distance;
    float max_distance;
    uint max_bounces;
}
