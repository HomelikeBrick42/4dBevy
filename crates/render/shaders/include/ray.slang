import material;

struct Ray
{
    float4 origin;
    float4 direction;
}

struct Hit
{
    float4 position;
    float4 normal;
    float distance;
    MaterialId material_id;
}
