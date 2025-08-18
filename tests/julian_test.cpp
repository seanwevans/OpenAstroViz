#include "julian.hpp"
#include <catch2/catch_test_macros.hpp>
#include <cmath>

TEST_CASE("julian_date_from_doy computes correct Julian date for known values") {
    REQUIRE(std::abs(julian::julian_date_from_doy(2000, 1, 0.5) - 2451545.0) < 1e-6);
    REQUIRE(std::abs(julian::julian_date_from_doy(2021, 275, 0.59097222) - 2459490.09097222) <
            1e-6);
}

TEST_CASE("doy_to_month_day handles valid and invalid day-of-year inputs") {
    int month = 0;
    int day = 0;

    SECTION("valid day-of-year returns correct month and day") {
        bool ok = julian::doy_to_month_day(2021, 275, month, day);
        REQUIRE(ok);
        REQUIRE(month == 10);
        REQUIRE(day == 2);
    }

    SECTION("valid leap day-of-year returns December 31") {
        bool ok = julian::doy_to_month_day(2020, 366, month, day);
        REQUIRE(ok);
        REQUIRE(month == 12);
        REQUIRE(day == 31);
    }

    SECTION("invalid day-of-year returns false") {
        bool ok = julian::doy_to_month_day(2021, 366, month, day);
        REQUIRE_FALSE(ok);
    }
}
