import material;
import ray;

struct HyperSphere
{
    float4 position;
    MaterialId material_id;
    float radius;

    Optional<Hit> intersect(Ray ray, float min_distance, float max_distance)
    {
        let oc = this.position - ray.origin;
        // TODO: can this be replaced with 1?
        let a = dot(ray.direction, ray.direction);
        let h = dot(ray.direction, oc);
        let c = dot(oc, oc) - this.radius * this.radius;
        let discriminant = h * h - a * c;

        if (discriminant < 0.0)
            return none;

        var hit : Hit;
        hit.distance = (h - sqrt(discriminant)) / a;
        if (hit.distance < min_distance || hit.distance > max_distance)
            return none;

        hit.position = ray.origin + ray.direction * hit.distance;
        hit.normal = (hit.position - this.position) / this.radius;
        hit.material_id = this.material_id;
        return hit;
    }
}
