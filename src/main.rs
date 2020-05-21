mod bookmarks;

use bookmarks::data::Bookmark;

fn main() {
    let test_string = String::from("<DT><A HREF=\"https://github.com/luvit/luv/blob/master/docs.md#uvspawnfile-options-onexit\" ADD_DATE=\"1578165853\" LAST_MODIFIED=\"1578165853\" TAGS=\"programming,vim\">luvit/luv</A>\n<DD>Bare libuv bindings for lua. Contribute to luvit/luv development by creating an account on GitHub.");
    Bookmark::from_html(&test_string);
}
