mod dynamic_to_one;
mod dynamic_to_many;

// TODO: Keep a database. The database is a bunch of dynamic junctions and 
// when you decide to swap in a value from outside to inspect it, it borrows 
// the dynamic mutably and uses the mem::swap() operation to put your new thing in
// where the old thing was
//
// (Likely the code to swap in your new value is inside of the underlying moogle type, 
//  to prevent the mutable ref from easily being kept around.)
//
// Ideally, the dynamic uses https://doc.rust-lang.org/stable/std/any/struct.TypeId.html
// to reject swaps that are not type-safe
//
// It can also use the type_name_of_id intrinsic to complain
//
// Possibly it accepts those swaps, but forces a recompile, at least of code that could
// have been using the struct in question. In those cases it should probably make processes
// that were running on that code panic with an error.