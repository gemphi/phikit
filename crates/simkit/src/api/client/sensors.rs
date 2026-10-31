use super::RpcClient;

use crate::common::{DistanceSensorData, LidarData};
use crate::sensors::{BarometerOutput, GpsOutput, ImuOutput, MagnetometerOutput};
use crate::Result;
impl RpcClient {
    /// API Endpoint
    pub fn get_lidar_data(&mut self, lidar_name: &str, vehicle_name: &str) -> Result<LidarData> {
        self.block_on(async {
            self.send_request("getLidarData", (lidar_name, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn get_imu_data(&mut self, imu_name: &str, vehicle_name: &str) -> Result<ImuOutput> {
        self.block_on(async {
            self.send_request("getImuData", (imu_name, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn get_barometer_data(
        &mut self,
        barometer_name: &str,
        vehicle_name: &str,
    ) -> Result<BarometerOutput> {
        self.block_on(async {
            self.send_request("getBarometerData", (barometer_name, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn get_magnetometer_data(
        &mut self,
        magnetometer_name: &str,
        vehicle_name: &str,
    ) -> Result<MagnetometerOutput> {
        self.block_on(async {
            self.send_request("getMagnetometerData", (magnetometer_name, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn get_gps_data(&mut self, gps_name: &str, vehicle_name: &str) -> Result<GpsOutput> {
        self.block_on(async {
            self.send_request("getGpsData", (gps_name, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn get_distance_sensor_data(
        &mut self,
        distance_sensor_name: &str,
        vehicle_name: &str,
    ) -> Result<DistanceSensorData> {
        self.block_on(async {
            self.send_request(
                "getDistanceSensorData",
                (distance_sensor_name, vehicle_name),
            )
            .await
        })
    }

}
