(set! "topics-per-page" 20)
(set! "replies-per-page" 20)
(set! "auto-refresh-interval" 0)
(set! "key-sequence-timeout" 1000)
(set! "theme" "dark")
(set! "timestamp-format" "relative")

; Initial view: "topic-list" or "aggregate"
(set! "initial-view" "aggregate")

; Initial settings (only used when initial-view matches)
; For aggregate view: "tech", "creative", "play", "apple", "jobs", "deals", "city", "qna", "index"
(set! "initial-tab" "index")

; For topic-list view: any valid node name (e.g., "python", "programmer", "share")
(set! "initial-node" "python")

(bind "C-c" "quit-immediate")
(bind "q" "remove-from-history")
(bind "ESC" "remove-from-history")
(bind "l" "history-back")
(bind "r" "history-forward")
(bind "g" "refresh")
(bind "?" "show-help")
(bind "a" "go-to-aggregate")
(bind "m" "go-to-notifications")
(bind "u" "go-to-profile")
(bind "s" "go-to-node-select")

; Node selection mode keybindings
(define-key "mode-node-select" "n" "next-item")
(define-key "mode-node-select" "p" "previous-item")
(define-key "mode-node-select" "RET" "select-current-node")
(define-key "mode-node-select" "TAB" "toggle-completion-mode")
(define-key "mode-node-select" "ESC" "remove-from-history")

(define-key "view-topic-list" "t" "open-topic")
(define-key "view-topic-list" "RET" "open-topic")
(define-key "view-topic-list" "+" "load-more-topics")
(define-key "view-topic-list" "o" "open-in-browser")
(define-key "view-topic-list" "s" "select-node")

; Quick node switching - bind digit keys to switch-node
; The action uses the favorite_nodes list (1st node = key "1", 2nd node = key "2", etc.)
; Default favorite nodes: 1=python, 2=programmer, 3=share, 4=create, 5=jobs, 6=go, 7=rust, 8=javascript, 9=linux
(define-key "view-topic-list" "1" "switch-node")
(define-key "view-topic-list" "2" "switch-node")
(define-key "view-topic-list" "3" "switch-node")
(define-key "view-topic-list" "4" "switch-node")
(define-key "view-topic-list" "5" "switch-node")
(define-key "view-topic-list" "6" "switch-node")
(define-key "view-topic-list" "7" "switch-node")
(define-key "view-topic-list" "8" "switch-node")
(define-key "view-topic-list" "9" "switch-node")

(define-key "view-topic-detail" "t" "toggle-replies")
(define-key "view-topic-detail" "o" "open-in-browser")
(define-key "view-topic-detail" "f" "enter-link-mode")
(define-key "view-topic-detail" "w" "copy-to-clipboard")
(define-key "view-topic-detail" "+" "load-more-replies")

; Space for scrolling down
(define-key "view-topic-detail" "SPC" "scroll-down")

(define-key "mode-replies" "n" "next-reply")
(define-key "mode-replies" "p" "previous-reply")
(define-key "mode-replies" "+" "load-more-replies")
(define-key "mode-replies" "<" "first-item")
(define-key "mode-replies" ">" "last-item")

; Link selection mode - configure which key opens which link
; Default home row keys: a=1, o=2, e=3, u=4, i=5, d=6, h=7, t=8, n=9, s=10
(link-key "a" 1)
(link-key "o" 2)
(link-key "e" 3)
(link-key "u" 4)
(link-key "i" 5)
(link-key "d" 6)
(link-key "h" 7)
(link-key "t" 8)
(link-key "n" 9)
(link-key "s" 10)

; Bind keys to the link-select action
(define-key "mode-link-selection" "a" "link-select")
(define-key "mode-link-selection" "o" "link-select")
(define-key "mode-link-selection" "e" "link-select")
(define-key "mode-link-selection" "u" "link-select")
(define-key "mode-link-selection" "i" "link-select")
(define-key "mode-link-selection" "d" "link-select")
(define-key "mode-link-selection" "h" "link-select")
(define-key "mode-link-selection" "t" "link-select")
(define-key "mode-link-selection" "n" "link-select")
(define-key "mode-link-selection" "s" "link-select")
(define-key "mode-link-selection" "ESC" "exit-link-mode")

(define-key "view-aggregate" "RET" "open-aggregate-item")
(define-key "view-aggregate" "o" "open-in-browser")
(define-key "view-aggregate" "g" "refresh-aggregate")

; Tab switching - single action reads the triggering key
; Configure which key maps to which tab (default shown below)
; Available tabs: tech, creative, play, apple, jobs, deals, city, qna, index
(tab-key "t" "tech")
(tab-key "c" "creative")
(tab-key "k" "play")
(tab-key "a" "apple")
(tab-key "j" "jobs")
(tab-key "d" "deals")
(tab-key "y" "city")
(tab-key "z" "qna")
(tab-key "i" "index")

; Bind keys to the switch-tab action
(define-key "view-aggregate" "t" "switch-tab")
(define-key "view-aggregate" "c" "switch-tab")
(define-key "view-aggregate" "k" "switch-tab")
(define-key "view-aggregate" "a" "switch-tab")
(define-key "view-aggregate" "j" "switch-tab")
(define-key "view-aggregate" "d" "switch-tab")
(define-key "view-aggregate" "y" "switch-tab")
(define-key "view-aggregate" "z" "switch-tab")
(define-key "view-aggregate" "i" "switch-tab")
