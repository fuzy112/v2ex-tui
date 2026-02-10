;; V2EX TUI Default Configuration
;; Auto-generated on first run if ~/.config/v2ex/config.lisp doesn't exist

;; ============================================
;; General Settings
;; ============================================

(set! initial-view 'topic-list)
(set! topics-per-page 20)
(set! replies-per-page 20)
(set! auto-refresh-interval 0)  ; 0 = disabled
(set! key-sequence-timeout 1000)  ; milliseconds

;; ============================================
;; Favorite Nodes (1-9 quick access)
;; ============================================
(set-favorite-nodes
  '((python "Python")
    (programmer "程序员")
    (share "分享发现")
    (create "分享创造")
    (jobs "酷工作")
    (go "Go 编程语言")
    (rust "Rust 编程语言")
    (javascript "JavaScript")
    (linux "Linux")))

(set-quick-keys
  '(1 python 2 programmer 3 share
    4 create 5 jobs 6 go
    7 rust 8 javascript 9 linux))

;; ============================================
;; Browser
;; ============================================
;; Use #f for system default, or specify command:
;; (set-browser "firefox")
;; (set-browser '("firefox" "--new-tab"))
(set-browser #f)

;; ============================================
;; Theme
;; ============================================
(set-theme 'dark)
;; For custom theme, set theme to 'custom and define colors:
;; (set-custom-theme
;;   '((foreground "#ebdbb2")
;;     (background "#282828")
;;     (primary "#b8bb26")))

;; ============================================
;; Timestamp Format
;; ============================================
(set-timestamp-format 'relative)  ; or 'absolute
(set-absolute-time-format "%Y-%m-%d %H:%M")

;; ============================================
;; Image Display (experimental)
;; ============================================
(set-inline-images #f)
(set-image-protocol 'auto)

;; ============================================
;; Keymaps - Emacs/dired style navigation
;; ============================================

;; Global bindings (available everywhere)
(bind-global "C-c" 'quit-immediate)
(bind-global "q" 'remove-from-history)
(bind-global "Esc" 'remove-from-history)
(bind-global "l" 'history-back)
(bind-global "r" 'history-forward)
(bind-global "?" 'show-help)
(bind-global "g" 'refresh)
(bind-global "a" 'go-to-aggregate)
(bind-global "m" 'go-to-notifications)
(bind-global "u" 'go-to-profile)
(bind-global "s" 'go-to-node-select)

;; View: Topic List
(with-view 'topic-list
  (bind "n" 'next-topic)
  (bind "p" 'previous-topic)
  (bind "t" 'open-topic)
  (bind "Enter" 'open-topic)
  (bind "+" 'load-more-topics)
  (bind "o" 'open-in-browser)
  (bind "s" 'select-node)
  (bind "<" 'first-topic)
  (bind ">" 'last-topic)
  
  ;; Quick node switching (1-9)
  (bind "1" '(switch-node "python"))
  (bind "2" '(switch-node "programmer"))
  (bind "3" '(switch-node "share"))
  (bind "4" '(switch-node "create"))
  (bind "5" '(switch-node "jobs"))
  (bind "6" '(switch-node "go"))
  (bind "7" '(switch-node "rust"))
  (bind "8" '(switch-node "javascript"))
  (bind "9" '(switch-node "linux"))
  
  ;; Page navigation
  (bind "C-v" 'page-down)
  (bind "M-v" 'page-up))

;; View: Topic Detail
(with-view 'topic-detail
  (bind "t" 'toggle-replies)
  (bind "o" 'open-in-browser)
  (bind "f" 'enter-link-mode)
  (bind "w" 'copy-to-clipboard)
  (bind "N" 'next-topic)
  (bind "P" 'previous-topic)
  (bind "+" 'load-more-replies)
  (bind "g" 'refresh-topic)
  (bind "l" 'history-back)
  
  ;; Mode: Replies visible (minor mode)
  (with-mode 'replies
    (bind "n" 'next-reply)
    (bind "p" 'previous-reply)
    (bind "+" 'load-more-replies)
    (bind "<" 'first-reply)
    (bind ">" 'last-reply)
    (bind "w" 'copy-reply))
  
  ;; Mode: Link selection (modal)
  (with-mode 'link-selection
    ;; Home row keys for link shortcuts
    (bind "a" '(link-select "a"))
    (bind "o" '(link-select "o"))
    (bind "e" '(link-select "e"))
    (bind "u" '(link-select "u"))
    (bind "i" '(link-select "i"))
    (bind "d" '(link-select "d"))
    (bind "h" '(link-select "h"))
    (bind "t" '(link-select "t"))
    (bind "n" '(link-select "n"))
    (bind "s" '(link-select "s"))
    (bind "Esc" 'exit-link-mode)
    (bind "C-g" 'exit-link-mode)))

;; View: Notifications
(with-view 'notifications
  (bind "n" 'next-notification)
  (bind "p" 'previous-notification)
  (bind "Enter" 'open-notification)
  (bind "g" 'refresh-notifications))

;; View: Profile
(with-view 'profile
  (bind "g" 'refresh-profile))

;; View: Aggregate
(with-view 'aggregate
  (bind "n" 'next-item)
  (bind "p" 'previous-item)
  (bind "Enter" 'open-item)
  (bind "o" 'open-in-browser)
  (bind "g" 'refresh-aggregate)
  ;; Tab switching
  (bind "t" '(switch-tab "tech"))
  (bind "c" '(switch-tab "creative"))
  (bind "k" '(switch-tab "play"))
  (bind "a" '(switch-tab "apple"))
  (bind "j" '(switch-tab "jobs"))
  (bind "d" '(switch-tab "deals"))
  (bind "y" '(switch-tab "city"))
  (bind "z" '(switch-tab "qna"))
  (bind "i" '(switch-tab "index")))

;; View: Node Select
(with-view 'node-select
  (bind "n" 'next-node)
  (bind "p" 'previous-node)
  (bind "Enter" 'select-node)
  (bind "Tab" 'toggle-completion-mode))

;; View: Help
(with-view 'help
  (bind "q" 'remove-from-history)
  (bind "Esc" 'remove-from-history)
  (bind "l" 'history-back))
