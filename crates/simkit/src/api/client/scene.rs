use super::RpcClient;
use crate::Result;
impl RpcClient {

    // ==================== SCENE/SEGMENTATION ====================

    /// API Endpoint
    pub fn sim_set_segmentation_object_id(
        &mut self,
        object_name: &str,
        object_id: i32,
        is_name_regex: bool,
    ) -> Result<bool> {
        self.block_on(async {
            self.send_request(
                "simSetSegmentationObjectID",
                (object_name, object_id, is_name_regex),
            )
            .await
        })

    /// API Endpoint
    pub fn sim_get_segmentation_object_id(&mut self, object_name: &str) -> Result<i32> {
        self.block_on(async {
            self.send_request("simGetSegmentationObjectID", (object_name,))
                .await
        })

    /// API Endpoint
    pub fn sim_swap_segmentation_objects(&mut self, object_id1: i32, object_id2: i32) -> Result<bool> {
        self.block_on(async {
            self.send_request("simSwapSegmentationObjects", (object_id1, object_id2))
                .await
        })

    /// API Endpoint
    pub fn sim_set_time_of_day_simple(&mut self, time_of_day: &str, is_dst: bool) -> Result<()> {
        self.block_on(async {
            // Simplified version - call the full method with sensible defaults
            // Format: "HH:MM:SS" or "YYYY-MM-DD HH:MM:SS"
            let full_datetime = if time_of_day.contains('-') {
                time_of_day.to_string()
            } else {
                format!("2024-01-01 {}", time_of_day)
            };
            self.send_request(
                "simSetTimeOfDay",
                (true, full_datetime, is_dst, 1.0, 60.0, true),
            )
            .await
        })
}
