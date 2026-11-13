# vbulletin

A block that recreates a vBulletin 3.8 message board in its default style:
the blue-gray tables with gradient category rows, Verdana throughout, the
navbar with breadcrumbs and the welcome box, popup menus, page navigation,
postbits with avatars and signatures, and the redirect pages. It is a
stress test of how far `tern.ui.el` and a plugin stylesheet go: every page
is plain HTML elements styled by `vbulletin.css`, with images shipped in
`img/` and drawn through CSS `url()`.

Pages: the forum home (categories, What's Going On?, statistics), forums
(sub-forums, sticky threads, sortable columns, Display Options, Forum Jump),
threads (Thread Tools, Search this Thread, Rate Thread, Quick Reply,
Similar Threads, printable version), Search with its options and results as
threads or posts (search words highlighted in the thread), New Posts,
Today's Posts, the Members List and profiles, Who's Online, the FAQ, the
Calendar, the User CP with subscriptions, and the Post New Thread and Reply
pages with the BB code toolbar, smilies, post icons and Preview.

The board is seeded from `seed.luau` (a Tern community, about 60 threads)
and you post as your login name. What you add (posts, threads, read marks,
votes, subscriptions) is kept in the plugin's `tern.kv`, so every block of
the plugin shows the same board and it survives restarts.

- **New vBulletin block** in the palette opens the board.
- `Tab` moves between fields, `Enter` submits a search, `⌘Enter` posts,
  `Esc` leaves a field or closes a menu.
- `/` opens the navbar search, `h` goes home, `Backspace` goes back.
- In a thread `j`/`k` move between posts, `r` replies; `n`/`p` turn pages
  in threads, forums, results and the Members List.

## Layout

| File | Holds |
| --- | --- |
| `host.luau` | The block: routes, keys, forms, focus and scrolling |
| `lib/db.luau` | The board: seed plus your additions, counts, unread state, search |
| `lib/chrome.luau` | Header, navbar, navigation buttons and footer |
| `lib/parts.luau` | Shared templates: forumbit, threadbit, pagenav, menus, selects |
| `lib/pages/*.luau` | One module per page |
| `lib/bbcode.luau` | BB code and smilies to nodes |
| `lib/field.luau` | Text fields the block edits itself |
| `vbulletin.css` | The vBulletin 3 style, plus one class per image |

## Install

```sh
tern plugin link tern-sdk/plugins/examples/vbulletin
```
