// Tests relying on Catch2's provided main, linked via -lCatch2Main.
#include <catch2/catch_test_macros.hpp>

#include <cmath>
#include "julian.hpp"

TEST_CASE("julian_date_from_doy computes expected Julian dates") {
    double jd1 = julian::julian_date_from_doy(2000, 1, 0.5);
    REQUIRE(std::abs(jd1 - 2451545.0) < 1e-6);

    double jd2 = julian::julian_date_from_doy(2021, 275, 0.59097222);
    REQUIRE(std::abs(jd2 - 2459490.09097222) < 1e-6);
}

TEST_CASE("doy_to_month_day returns false for invalid day of year") {
    int month, day;
    bool ok = julian::doy_to_month_day(2021, 366, month, day);
    REQUIRE_FALSE(ok);
}
