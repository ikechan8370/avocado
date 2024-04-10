#[cfg(test)]
mod tests {
    use crate::utils::time::format_duration;

    #[test]
    fn it_works() {
        assert_eq!(format_duration(0).unwrap(), "0秒");
        assert_eq!(format_duration(1).unwrap(), "1秒");
        assert_eq!(format_duration(60).unwrap(), "1分钟");
        assert_eq!(format_duration(61).unwrap(), "1分钟1秒");
        assert_eq!(format_duration(3600).unwrap(), "1小时");
        assert_eq!(format_duration(3601).unwrap(), "1小时1秒");
        assert_eq!(format_duration(3660).unwrap(), "1小时1分钟");
        assert_eq!(format_duration(3661).unwrap(), "1小时1分钟1秒");
        assert_eq!(format_duration(86400).unwrap(), "1天");
        assert_eq!(format_duration(86401).unwrap(), "1天1秒");
        assert_eq!(format_duration(86460).unwrap(), "1天1分钟");
        assert_eq!(format_duration(86461).unwrap(), "1天1分钟1秒");
        assert_eq!(format_duration(90061).unwrap(), "1天1小时1分钟1秒");
    }
}
