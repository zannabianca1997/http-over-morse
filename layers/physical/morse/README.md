# Encoding

Crate containing the morse encoding data

## LUT generation 

Most of the decoding is made using lookup tables, based on a precalculated
perfect hash. I did not bother optimizing neither the hash function not the
search, so finding the values takes ~ 40min.

To avoid spending that at each compilation, the values are cached inside
`.phcache`. To check them, set the `CALC_HASHER` env var and look at your cores
go :3
