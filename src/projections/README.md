## Usage example for the projections

The `projection trait` has different methods, but only two should be used outside .
The `forward`, in which you convert geographic coordinates to cartesian coordinates and the `inverse` in which you do the opposite.

To call both these methods, with the projection to be used on a grid, you need three parameters:
- The latitude and longitude coordinates you want to project.
- The polyhedron or 3D shape to use (icosahedron, dodecahedron, etc).
- The 2D flat configuration.

The projection is independent of the polyhedron, this mean that it allows utilization of another polyhedron on that same projection.

Example:

```rust
let position = PositionGeo {
    lat: 38.695125,
    lon: -9.222154,
};

// The projection which will be used
let projection = Vgc;

let result = projection.forward(vec![position], Some(&Icosahedron {}), &IcosahedronNet {});
```

The inverse will share the same behiaviour.

Each projection will be assigned a specific file.

## Things to consider
- The `trait` has authalic to geodetic conversion (and vice-versa), so the project is quite accurate in terms of representating spatial relationships from an equal-area standpoint. The methods `lat_authalic_to_geodetic`, `lat_geodetic_to_authalic`, `fourier_coefficients`, `apply_clenshaw_summation` are all used in these process, being that the first two are the ones called for the conversion.
- There is a method called `to_3d` which allows calculating a 3D unit vector from a latitude/longitude of a given point P.