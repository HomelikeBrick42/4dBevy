struct Rotor
{
    float s;
    float e12;
    float e13;
    float e14;
    float e23;
    float e24;
    float e34;
    float e1234;

    Rotor then(Rotor other)
    {
        let a1 = this.s;
        let b1 = this.e12;
        let c1 = this.e13;
        let d1 = this.e14;
        let e1 = this.e23;
        let f1 = this.e24;
        let g1 = this.e34;
        let h1 = this.e1234;
        let a2 = other.s;
        let b2 = other.e12;
        let c2 = other.e13;
        let d2 = other.e14;
        let e2 = other.e23;
        let f2 = other.e24;
        let g2 = other.e34;
        let h2 = other.e1234;
        var result : Rotor;
        result.s = -b1 * b2 + -c1 * c2 + -d1 * d2 + -e1 * e2 + -f1 * f2 + -g1 * g2 + a1 * a2 + h1 * h2;
        result.e12 = -c1 * e2 + -d1 * f2 + -g1 * h2 + -g2 * h1 + a1 * b2 + a2 * b1 + c2 * e1 + d2 * f1;
        result.e13 = -b2 * e1 + -d1 * g2 + a1 * c2 + a2 * c1 + b1 * e2 + d2 * g1 + f1 * h2 + f2 * h1;
        result.e14 = -b2 * f1 + -c2 * g1 + -e1 * h2 + -e2 * h1 + a1 * d2 + a2 * d1 + b1 * f2 + c1 * g2;
        result.e23 = -b1 * c2 + -d1 * h2 + -d2 * h1 + -f1 * g2 + a1 * e2 + a2 * e1 + b2 * c1 + f2 * g1;
        result.e24 = -b1 * d2 + -e2 * g1 + a1 * f2 + a2 * f1 + b2 * d1 + c1 * h2 + c2 * h1 + e1 * g2;
        result.e34 = -b1 * h2 + -b2 * h1 + -c1 * d2 + -e1 * f2 + a1 * g2 + a2 * g1 + c2 * d1 + e2 * f1;
        result.e1234 = -c1 * f2 + -c2 * f1 + a1 * h2 + a2 * h1 + b1 * g2 + b2 * g1 + d1 * e2 + d2 * e1;
        return result;
    }

    float4 rotate(float4 direction)
    {
        let a = this.s;
        let b = this.e12;
        let c = this.e13;
        let d = this.e14;
        let e = this.e23;
        let f = this.e24;
        let g = this.e34;
        let h = this.e1234;
        let p3 = direction.x;
        let p2 = direction.y;
        let p1 = direction.z;
        let p0 = direction.w;
        let ap2 = a * p2;
        let bp3 = b * p3;
        let ep1 = e * p1;
        let fp0 = f * p0;
        let ap3 = a * p3;
        let bp2 = b * p2;
        let cp1 = c * p1;
        let dp0 = d * p0;
        let ap1 = a * p1;
        let gp0 = g * p0;
        let cp3 = c * p3;
        let ep2 = e * p2;
        let ap0 = a * p0;
        let gp1 = g * p1;
        let dp3 = d * p3;
        let fp2 = f * p2;
        let s0 = ep1 - ap2 - bp3 - fp0;
        let s1 = ap3 + cp1 - bp2 - dp0;
        let s2 = ap1 + ep2 - gp0 - cp3;
        let s3 = fp2 - ap0 - gp1 - dp3;
        let new_direction = float4(
            p0 + 2.0 * (h * (b * p1 + c * p2 + e * p3 - h * p0) + f * s0 + d * s1 + g * s2),
            p1 + 2.0 * (h * (d * p2 + f * p3 - h * p1 - b * p0) + g * s3 - e * s0 - c * s1),
            p2 + 2.0 * (h * (g * p3 - h * p2 - c * p0 - d * p1) + b * s1 - f * s3 - e * s2),
            p3 + 2.0 * (d * s3 + c * s2 + b * s0 - h * (g * p2 + h * p3 + e * p0 + f * p1)));
        return new_direction.wzyx;
    }
}
