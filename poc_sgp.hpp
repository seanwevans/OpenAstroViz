#pragma once
#include <string>
#include <cmath>

inline constexpr double kPi = 3.14159265358979323846;
inline constexpr double kEarthMu = 3.986004418e14;  // m^3 s^-2
inline constexpr double kEarthRad = 6378.137e3;     // m
inline constexpr double deg2rad(double d) { return d * kPi / 180.0; }

struct Tle {
    std::string line1;
    std::string line2;
};

struct Orbit {
    double epoch_jd;      // Julian Date
    double inc;           // inclination [rad]
    double raan;          // right asc. node [rad]
    double ecc;           // eccentricity
    double argp;          // argument of perigee [rad]
    double mean_anom;     // mean anomaly [rad]
    double mean_motion;   // rad/s
};

Orbit parse_tle(const Tle &tle);
void propagate(const Orbit &o, double dt, double &x, double &y, double &z);

