// test compile
// test run

bvec2 main() {
    bvec3 v = bvec3(true, false, true);
    return v.xz;  // Should return bvec2(true, true)
}

// run: == bvec2(true, true)



