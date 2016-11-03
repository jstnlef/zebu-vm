from test_milestones import get_fncptr


def test_trunc():
    fn = get_fncptr("test_trunc", "test_fnc")
    assert fn() == 0x58324b55


def test_sext():
    fn = get_fncptr("test_sext", "test_fnc")
    assert fn() == 0xffffffffa8324b55

def test_zext():
    fn = get_fncptr("test_zext", "test_fnc")
    assert fn() == 0x00000000a8324b55
