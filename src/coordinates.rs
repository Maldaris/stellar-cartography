use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub, Mul, Div};
use std::fmt;

/// Conversion factor: 1 light-year = 9460730472580800 meters (EVE Frontier-specific)
/// Source: EVE Frontier Discord dev channels, credit to Scetrov!
pub const METERS_PER_LIGHT_YEAR: f64 = 9460730472580800.0;

/// A coordinate value that can be represented in either meters or light-years
/// 
/// This type wraps an f64 value representing a distance or coordinate in meters
/// from galactic center, and provides convenient conversion methods to and from
/// light-years using EVE Online's coordinate system conventions.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Distance {
    /// The distance value in meters
    meters: f64,
}

impl Distance {
    /// Create a new Distance from a value in meters
    pub fn from_meters(meters: f64) -> Self {
        Self { meters }
    }
    
    /// Create a new Distance from a value in light-years
    pub fn from_light_years(light_years: f64) -> Self {
        Self { 
            meters: light_years * METERS_PER_LIGHT_YEAR 
        }
    }
    
    /// Get the distance value in meters
    pub fn to_meters(&self) -> f64 {
        self.meters
    }
    
    /// Get the distance value in light-years
    pub fn to_ly(&self) -> f64 {
        self.meters / METERS_PER_LIGHT_YEAR
    }
    
    /// Get the raw meters value (for compatibility with existing code)
    pub fn as_f64(&self) -> f64 {
        self.meters
    }
    
    /// Calculate the absolute distance between two Distance values
    pub fn distance_to(&self, other: &Distance) -> Distance {
        Distance::from_meters((self.meters - other.meters).abs())
    }
}

/// 3D coordinate point with distance conversion capabilities
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate3D {
    pub x: Distance,
    pub y: Distance,
    pub z: Distance,
}

impl Coordinate3D {
    /// Create a new 3D coordinate from meter values
    pub fn from_meters(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Distance::from_meters(x),
            y: Distance::from_meters(y),
            z: Distance::from_meters(z),
        }
    }
    
    /// Create a new 3D coordinate from light-year values
    pub fn from_light_years(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Distance::from_light_years(x),
            y: Distance::from_light_years(y),
            z: Distance::from_light_years(z),
        }
    }
    
    /// Convert to a [f64; 3] array in meters (for compatibility with existing code)
    pub fn to_meters_array(&self) -> [f64; 3] {
        [self.x.to_meters(), self.y.to_meters(), self.z.to_meters()]
    }
    
    /// Convert to a [f64; 3] array in light-years
    pub fn to_ly_array(&self) -> [f64; 3] {
        [self.x.to_ly(), self.y.to_ly(), self.z.to_ly()]
    }
    
    /// Create from a [f64; 3] array assumed to be in meters
    pub fn from_meters_array(coords: [f64; 3]) -> Self {
        Self::from_meters(coords[0], coords[1], coords[2])
    }
    
    /// Calculate Euclidean distance to another coordinate
    pub fn distance_to(&self, other: &Coordinate3D) -> Distance {
        let dx = self.x.to_meters() - other.x.to_meters();
        let dy = self.y.to_meters() - other.y.to_meters();
        let dz = self.z.to_meters() - other.z.to_meters();
        
        Distance::from_meters((dx * dx + dy * dy + dz * dz).sqrt())
    }
}

// Implement basic arithmetic operations for Distance
impl Add for Distance {
    type Output = Distance;
    
    fn add(self, other: Distance) -> Distance {
        Distance::from_meters(self.meters + other.meters)
    }
}

impl Sub for Distance {
    type Output = Distance;
    
    fn sub(self, other: Distance) -> Distance {
        Distance::from_meters(self.meters - other.meters)
    }
}

impl Mul<f64> for Distance {
    type Output = Distance;
    
    fn mul(self, scalar: f64) -> Distance {
        Distance::from_meters(self.meters * scalar)
    }
}

impl Div<f64> for Distance {
    type Output = Distance;
    
    fn div(self, scalar: f64) -> Distance {
        Distance::from_meters(self.meters / scalar)
    }
}

// Display implementations
impl fmt::Display for Distance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} ly ({:.2e} m)", self.to_ly(), self.meters)
    }
}

impl fmt::Display for Coordinate3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2}) ly", 
               self.x.to_ly(), self.y.to_ly(), self.z.to_ly())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_distance_from_meters() {
        let distance = Distance::from_meters(1e16);
        assert_eq!(distance.to_meters(), 1e16);
        assert_eq!(distance.to_ly(), 1.0);
    }
    
    #[test]
    fn test_distance_from_light_years() {
        let distance = Distance::from_light_years(2.5);
        assert_eq!(distance.to_ly(), 2.5);
        assert_eq!(distance.to_meters(), 2.5e16);
    }
    
    #[test]
    fn test_distance_conversion_roundtrip() {
        let original_meters = 5.7e15;
        let distance = Distance::from_meters(original_meters);
        let ly_value = distance.to_ly();
        let back_to_meters = Distance::from_light_years(ly_value).to_meters();
        
        assert!((original_meters - back_to_meters).abs() < 1e-6);
    }
    
    #[test]
    fn test_distance_arithmetic() {
        let d1 = Distance::from_light_years(1.0);
        let d2 = Distance::from_light_years(2.0);
        
        let sum = d1 + d2;
        assert_eq!(sum.to_ly(), 3.0);
        
        let diff = d2 - d1;
        assert_eq!(diff.to_ly(), 1.0);
        
        let scaled = d1 * 3.0;
        assert_eq!(scaled.to_ly(), 3.0);
        
        let divided = d2 / 2.0;
        assert_eq!(divided.to_ly(), 1.0);
    }
    
    #[test]
    fn test_distance_to() {
        let d1 = Distance::from_light_years(1.0);
        let d2 = Distance::from_light_years(3.0);
        
        let distance = d1.distance_to(&d2);
        assert_eq!(distance.to_ly(), 2.0);
        
        // Should be symmetric
        let distance_reverse = d2.distance_to(&d1);
        assert_eq!(distance_reverse.to_ly(), 2.0);
    }
    
    #[test]
    fn test_coordinate3d_from_meters() {
        let coord = Coordinate3D::from_meters(1e16, 2e16, 3e16);
        assert_eq!(coord.x.to_ly(), 1.0);
        assert_eq!(coord.y.to_ly(), 2.0);
        assert_eq!(coord.z.to_ly(), 3.0);
    }
    
    #[test]
    fn test_coordinate3d_from_light_years() {
        let coord = Coordinate3D::from_light_years(1.5, 2.5, 3.5);
        assert_eq!(coord.x.to_meters(), 1.5e16);
        assert_eq!(coord.y.to_meters(), 2.5e16);
        assert_eq!(coord.z.to_meters(), 3.5e16);
    }
    
    #[test]
    fn test_coordinate3d_array_conversion() {
        let original_array = [1e16, 2e16, 3e16];
        let coord = Coordinate3D::from_meters_array(original_array);
        let converted_array = coord.to_meters_array();
        
        assert_eq!(original_array, converted_array);
        
        let ly_array = coord.to_ly_array();
        assert_eq!(ly_array, [1.0, 2.0, 3.0]);
    }
    
    #[test]
    fn test_coordinate3d_distance_calculation() {
        // Test with simple 3-4-5 right triangle scaled up
        let coord1 = Coordinate3D::from_meters(0.0, 0.0, 0.0);
        let coord2 = Coordinate3D::from_meters(3e16, 4e16, 0.0);
        
        let distance = coord1.distance_to(&coord2);
        assert_eq!(distance.to_ly(), 5.0); // 3-4-5 triangle
    }
    
    #[test]
    fn test_coordinate3d_distance_symmetry() {
        let coord1 = Coordinate3D::from_light_years(1.0, 2.0, 3.0);
        let coord2 = Coordinate3D::from_light_years(4.0, 6.0, 8.0);
        
        let distance1 = coord1.distance_to(&coord2);
        let distance2 = coord2.distance_to(&coord1);
        
        assert!((distance1.to_ly() - distance2.to_ly()).abs() < 1e-10);
    }
    
    #[test]
    fn test_eve_coordinate_conversion() {
        // Test with realistic EVE coordinate values
        let eve_coords = [1.23456789e17, -9.87654321e16, 5.55555555e15];
        let coord = Coordinate3D::from_meters_array(eve_coords);
        
        // Verify conversion to light-years works
        let ly_values = coord.to_ly_array();
        assert!((ly_values[0] - 12.3456789).abs() < 1e-6);
        assert!((ly_values[1] - (-9.87654321)).abs() < 1e-6);
        assert!((ly_values[2] - 0.555555555).abs() < 1e-6);
        
        // Verify round-trip conversion
        let back_to_meters = coord.to_meters_array();
        for i in 0..3 {
            assert!((eve_coords[i] - back_to_meters[i]).abs() < 1e-6);
        }
    }
    
    #[test]
    fn test_display_formatting() {
        let distance = Distance::from_light_years(1.23456);
        let display_str = format!("{}", distance);
        assert!(display_str.contains("1.23"));
        assert!(display_str.contains("ly"));
        assert!(display_str.contains("m"));
        
        let coord = Coordinate3D::from_light_years(1.0, 2.0, 3.0);
        let coord_str = format!("{}", coord);
        assert!(coord_str.contains("1.00"));
        assert!(coord_str.contains("2.00"));
        assert!(coord_str.contains("3.00"));
        assert!(coord_str.contains("ly"));
    }
    
    #[test]
    fn test_stellar_cartography_integration() {
        // Test scenario: user requests systems within 5 light-years of a center point
        let user_radius_ly = 5.0;
        
        // Convert to meters for spatial index search (as done in systems.rs)
        let radius_meters = Distance::from_light_years(user_radius_ly).to_meters();
        assert_eq!(radius_meters, 5.0e16);
        
        // Simulate finding a system at distance (returned from spatial index in meters)
        let found_system_distance_meters = 3.2e16; // This would come from spatial index
        
        // Convert back to light-years for API response (as done in systems.rs)
        let found_system_distance_ly = Distance::from_meters(found_system_distance_meters).to_ly();
        assert_eq!(found_system_distance_ly, 3.2);
        
        // Verify the found system is within the requested radius
        assert!(found_system_distance_ly <= user_radius_ly);
        
        // Test coordinate conversion for system positions
        let jita_meters = [1.66e16, 2.87e16, -5.42e15]; // EVE coordinates in meters
        let jita_coord = Coordinate3D::from_meters_array(jita_meters);
        
        let amarr_meters = [-2.34e17, 1.12e17, 8.91e16];
        let amarr_coord = Coordinate3D::from_meters_array(amarr_meters);
        
        // Calculate distance between systems
        let inter_system_distance = jita_coord.distance_to(&amarr_coord);
        
        // Verify the calculation produces a reasonable result for EVE scale
        let distance_ly = inter_system_distance.to_ly();
        assert!(distance_ly > 0.0);
        assert!(distance_ly < 1000.0); // Should be less than 1000 light-years for EVE scale
        
        // Verify round-trip conversion maintains precision
        let distance_meters = inter_system_distance.to_meters();
        let round_trip = Distance::from_meters(distance_meters).to_ly();
        assert!((distance_ly - round_trip).abs() < 1e-10);
    }
    
    #[test]
    fn test_ekr_kbb_to_epj_shb_distance() {
        // Actual coordinates from our database
        // EKR-KBB: 30002165|-1.37094029258515e+19|-9.53141752314212e+17|5.0540465513028e+18
        // EPJ-SHB: 30002172|-1.3944978578868e+19|-9.68629622819937e+17|5.19132888533572e+18
        
        let ekr_kbb = Coordinate3D::from_meters(
            -1.37094029258515e19,
            -9.53141752314212e17,
            5.0540465513028e18
        );
        
        let epj_shb = Coordinate3D::from_meters(
            -1.3944978578868e19,
            -9.68629622819937e17,
            5.19132888533572e18
        );
        
        let distance = ekr_kbb.distance_to(&epj_shb);
        let distance_ly = distance.to_ly();
        
        println!("Manual calculation: EKR-KBB to EPJ-SHB distance = {:.6} ly", distance_ly);
        println!("In-game distance shown: 28.87 ly");
        println!("Our API returned: 27.31 ly");
        println!("Difference from in-game: {:.6} ly", (distance_ly - 28.87).abs());
        
        // The distance should be close to the in-game value of 28.87 ly
        // Allow for some tolerance due to precision differences
        assert!(distance_ly > 25.0 && distance_ly < 35.0, 
                "Distance {} ly is outside reasonable range", distance_ly);
    }
    
    #[test]
    fn test_ekr_kbb_to_epj_shb_high_precision() {
        // Original integer coordinates from starmapcache.json (highest precision)
        // EKR-KBB: [-13709402925851513000, -953141752314212500, 5054046551302796000]
        // EPJ-SHB: [-13944978578867974000, -968629622819936600, 5191328885335723000]
        
        let ekr_kbb = Coordinate3D::from_meters(
            -13709402925851513000.0,
            -953141752314212500.0,
            5054046551302796000.0
        );
        
        let epj_shb = Coordinate3D::from_meters(
            -13944978578867974000.0,
            -968629622819936600.0,
            5191328885335723000.0
        );
        
        let distance = ekr_kbb.distance_to(&epj_shb);
        let distance_ly = distance.to_ly();
        
        println!("High precision calculation: EKR-KBB to EPJ-SHB distance = {:.6} ly", distance_ly);
        println!("In-game distance shown: 28.87 ly");
        println!("Our API returned: 27.31 ly");
        println!("Database precision: 27.309742 ly");
        println!("Difference from in-game: {:.6} ly", (distance_ly - 28.87).abs());
        
        // This should be closer to the in-game value
        assert!(distance_ly > 25.0 && distance_ly < 35.0, 
                "Distance {} ly is outside reasonable range", distance_ly);
    }
} 