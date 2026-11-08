# Morse enumerations

Enumerations for morse types

## Generation

Compiling this crate will build the lookup tables for symbol digest. The table
are built with an extensive search over the entire 16 bit space. While it is
parallelized over all cores, it can take some time.

To ease this, the search results are cached inside `.phcache`, that's committed
to the repo. Deleting it runs the search again, but it's rarely needed, as they
are keyed by the full mapping that they represent (change in the mapped function
run a new search without having to delete the old result).
