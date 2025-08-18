#include <catch2/catch_all.hpp>

#include "julian.hpp"

TEST_CASE("Julian date matches known references", "[julian][doy]") {
    double jd1 = julian::julian_date_from_doy(2000, 1, 0.5);
    REQUIRE(jd1 == Catch::Approx(2451545.0).margin(1e-6));

    double jd2 = julian::julian_date_from_doy(2021, 275, 0.59097222);
    REQUIRE(jd2 == Catch::Approx(2459490.09097222).margin(1e-6));

    double jd3 = julian::julian_date_from_calendar(1582, 10, 15, 0.0);
    REQUIRE(jd3 == Catch::Approx(2299160.5).margin(1e-6));
}

TEST_CASE("doy_to_month_day handles leap-year boundaries", "[doy_to_month_day][leap-year]") {
    int month = 0;
    int day = 0;

    REQUIRE(julian::doy_to_month_day(2020, 59, month, day));
    CHECK(month == 2);
    CHECK(day == 28);

    REQUIRE(julian::doy_to_month_day(2020, 60, month, day));
    CHECK(month == 2);
    CHECK(day == 29);

    REQUIRE(julian::doy_to_month_day(2020, 61, month, day));
    CHECK(month == 3);
    CHECK(day == 1);

    REQUIRE(julian::doy_to_month_day(2021, 59, month, day));
    CHECK(month == 2);
    CHECK(day == 28);

    REQUIRE(julian::doy_to_month_day(2021, 60, month, day));
    CHECK(month == 3);
    CHECK(day == 1);
}

TEST_CASE("doy_to_month_day rejects invalid DOYs", "[doy_to_month_day][invalid]") {
    int month = 0;
    int day = 0;

    CHECK_FALSE(julian::doy_to_month_day(2021, 366, month, day));
    CHECK_FALSE(julian::doy_to_month_day(2020, 0, month, day));
    CHECK_FALSE(julian::doy_to_month_day(2020, 367, month, day));
}

