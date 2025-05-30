import material;

struct Ray
{
    float4 origin;
    float4 direction;
}

struct Hit
{
    Material material;
    float distance;
    float4 position;
    float4 normal;
}
