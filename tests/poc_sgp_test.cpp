#include "poc_sgp.hpp"
#include <gtest/gtest.h>

TEST(ParseTLE, ISSExample) {
    Tle tle;
    tle.line1 = "1 25544U 98067A   25202.31751672  .00008283  00000+0  15302-3 0  9997";
    tle.line2 = "2 25544  51.6344 137.8967 0002535 105.6905 358.2995 15.49987077 52045";

    Orbit o = parse_tle(tle);

    EXPECT_NEAR(o.inc, deg2rad(51.6344), 1e-6);
    EXPECT_NEAR(o.raan, deg2rad(137.8967), 1e-6);
    EXPECT_NEAR(o.ecc, 0.0002535, 1e-7);
    EXPECT_NEAR(o.argp, deg2rad(105.6905), 1e-6);
    EXPECT_NEAR(o.mean_anom, deg2rad(358.2995), 1e-6);
    double mm = 15.49987077 * 2.0 * kPi / 86400.0;
    EXPECT_NEAR(o.mean_motion, mm, 1e-12);
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
