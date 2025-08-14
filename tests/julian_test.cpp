#include <catch2/catch_test_macros.hpp>
#include <cmath>

#include "julian.hpp"

TEST_CASE("julian_date_from_doy computes expected Julian date", "[julian]") {
    SECTION("J2000 epoch") {
        double jd1 = julian_date_from_doy(2000, 1, 0.5);
        REQUIRE(std::abs(jd1 - 2451545.0) < 1e-6);
    }

    SECTION("2021 day 275") {
        double jd2 = julian_date_from_doy(2021, 275, 0.59097222);
        REQUIRE(std::abs(jd2 - 2459490.09097222) < 1e-6);
    }
}
