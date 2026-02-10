;; Basic Configuration Example
;; ~/.config/v2ex/config.lisp

;; ============================================
;; Personal Settings
;; ============================================

;; Start in notifications view
(set! initial-view 'notifications)

;; Load 30 items per page
(set! topics-per-page 30)
(set! replies-per-page 30)

;; Use Firefox for links
(set-browser "firefox")

;; Dark theme
(set-theme 'dark)

;; Relative timestamps ("2 hours ago")
(set-timestamp-format 'relative)

;; ============================================
;; My Favorite Nodes
;; ============================================

(set-favorite-nodes
  '((rust "Rust")
    (go "Go 编程语言")
    (python "Python")
    (linux "Linux")
    (jobs "酷工作")))

(set-quick-keys
  '(1 rust 2 go 3 python 4 linux 5 jobs))

;; ============================================
;; Key Bindings
;; ============================================

;; Reload config quickly
(bind-global "C-c C-r" 'reload-config)

;; Topic list
(with-view 'topic-list
  (bind "j" 'next-topic)      ; Vim-style
  (bind "k" 'previous-topic)) ; Vim-style

;; Topic detail
(with-view 'topic-detail
  ;; Vim-style navigation
  (bind "j" 'next-reply)
  (bind "k" 'previous-reply)
  
  ;; Quick open in browser
  (bind "O" 'open-in-browser))
