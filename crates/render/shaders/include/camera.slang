struct Camera
{
    float4 position;
    float4 forward;
    float4 right;
    float4 up;
    float aspect;
    float min_distance;
    float max_distance;
    uint max_bounces;
}
