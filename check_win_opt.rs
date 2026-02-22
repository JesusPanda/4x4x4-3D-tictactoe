
// Optimized check_win using bitwise parallel checking
#[inline(always)]
fn check_win_optimized(mask: u64) -> bool {
    let p0 = mask as u16;
    let p1 = (mask >> 16) as u16;
    let p2 = (mask >> 32) as u16;
    let p3 = (mask >> 48) as u16;

    #[inline(always)]
    fn check_plane(m: u16) -> bool {
        let t_rows = m & (m >> 1);
        if (t_rows & (t_rows >> 2) & 0x1111) != 0 { return true; }

        let t_cols = m & (m >> 4);
        if (t_cols & (t_cols >> 8) & 0x000F) != 0 { return true; }

        let t_d1 = m & (m >> 5);
        if (t_d1 & (t_d1 >> 10) & 0x0001) != 0 { return true; }

        let t_d2 = m & (m >> 3);
        if (t_d2 & (t_d2 >> 6) & 0x0008) != 0 { return true; }

        false
    }

    if check_plane(p0) { return true; }
    if check_plane(p1) { return true; }
    if check_plane(p2) { return true; }
    if check_plane(p3) { return true; }

    if (p0 & p1 & p2 & p3) != 0 { return true; }

    if (p0 & (p1 >> 1) & (p2 >> 2) & (p3 >> 3) & 0x1111) != 0 { return true; }
    if (p0 & (p1 << 1) & (p2 << 2) & (p3 << 3) & 0x8888) != 0 { return true; }
    if (p0 & (p1 >> 4) & (p2 >> 8) & (p3 >> 12) & 0x000F) != 0 { return true; }
    if (p0 & (p1 << 4) & (p2 << 8) & (p3 << 12) & 0xF000) != 0 { return true; }

    if (p0 & (p1 >> 5) & (p2 >> 10) & (p3 >> 15) & 1) != 0 { return true; }
    if (p0 & (p1 >> 3) & (p2 >> 6) & (p3 >> 9) & 8) != 0 { return true; }
    if (p0 & (p1 << 3) & (p2 << 6) & (p3 << 9) & 0x1000) != 0 { return true; }
    if (p0 & (p1 << 5) & (p2 << 10) & (p3 << 15) & 0x8000) != 0 { return true; }

    false
}
