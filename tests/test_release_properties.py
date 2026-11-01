"""Generated release decisions use independent numeric expectations."""
from hypothesis import given, settings, strategies as st
from tools.release import next_version, version_tuple

versions = st.tuples(st.integers(0,1000),st.integers(0,1000),st.integers(0,1000))


@settings(max_examples=256)
@given(floor=versions, history=st.lists(versions,max_size=32))
def test_next_version_never_reuses_history_or_falls_below_floor(floor, history):
    render = lambda v: '.'.join(map(str,v))
    selected = next_version(render(floor), list(map(render,history)))
    assert selected >= floor
    assert version_tuple(render(selected)) == selected
    if history:
        assert selected > max(history)
    else:
        assert selected == floor
