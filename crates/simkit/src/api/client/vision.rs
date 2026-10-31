use super::RpcClient;

use crate::common::{CameraInfo, Pose, ImageRequest, ImageResponse, ImageType};
use crate::Result;
impl RpcClient {
    /// API Endpoint
    pub fn sim_get_camera_info(&mut self, camera_name: &str, vehicle_name: &str) -> Result<CameraInfo> {
        self.block_on(async {
            self.send_request("simGetCameraInfo", (camera_name, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn sim_set_camera_pose(
        &mut self,
        camera_name: &str,
        pose: Pose,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("simSetCameraPose", (camera_name, pose, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn sim_set_camera_fov(
        &mut self,
        camera_name: &str,
        fov_degrees: f32,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("simSetCameraFov", (camera_name, fov_degrees, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn sim_get_images(
        &mut self,
        requests: Vec<ImageRequest>,
        vehicle_name: &str,
    ) -> Result<Vec<ImageResponse>> {
        self.block_on(async {
            self.send_request("simGetImages", (requests, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn sim_get_image(
        &mut self,
        camera_name: &str,
        image_type: ImageType,
        vehicle_name: &str,
    ) -> Result<Vec<u8>> {
        self.block_on(async {
            self.send_request("simGetImage", (camera_name, image_type as u8, vehicle_name))
                .await
        })
    }

}
