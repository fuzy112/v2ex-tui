;; Advanced Configuration Example
;; ~/.config/v2ex/config.lisp

;; ============================================
;; Platform-Specific Settings
;; ============================================

(when (eq system-type 'macos)
  ;; macOS-specific browser
  (set-browser "open")
  
  ;; macOS specific key
  (bind-global "M-o" 'open-in-browser))

(when (eq system-type 'linux)
  ;; Check for specific terminal
  (when (string=? (getenv "TERM") "xterm-kitty")
    (set-image-protocol 'kitty)
    (set-inline-images #t)))

;; ============================================
;; Custom Functions
;; ============================================

;; Quick navigation to first unread
(define (goto-first-unread)
  (first-topic)
  ;; Custom logic could go here
  (open-topic))

;; Smart load: load more if at end, else just move
(define (smart-next)
  (if (at-last-topic?)
      (load-more-topics)
      (next-topic)))

;; Open and mark as read (custom workflow)
(define (open-and-mark)
  (open-topic)
  ;; Future: mark as read functionality
  )

;; ============================================
;; Complex Keybindings
;; ============================================

;; Leader key style (like Vim)
(define-key "SPC t" 'toggle-replies)
(define-key "SPC o" 'open-in-browser)
(define-key "SPC r" 'refresh)
(define-key "SPC n" 'go-to-notifications)
(define-key "SPC p" 'go-to-profile)
(define-key "SPC a" 'go-to-aggregate)

;; Quick save/load bookmarks (future feature)
(define-key "C-c b s" 'save-bookmark)
(define-key "C-c b l" 'load-bookmark)
(define-key "C-c b b" 'list-bookmarks)

;; ============================================
;; Mode-Specific Configurations
;; ============================================

(with-view 'topic-detail
  ;; Custom link selection with additional keys
  (with-mode 'link-selection
    ;; Numbers for quick access
    (bind "1" '(link-select "a"))
    (bind "2" '(link-select "o"))
    (bind "3" '(link-select "e"))
    ;; Keep Esc to exit
    (bind "Esc" 'exit-link-mode))
  
  ;; Enhanced replies mode
  (with-mode 'replies
    ;; Vote up/down (future feature)
    (bind "+" 'upvote-reply)
    (bind "-" 'downvote-reply)
    ;; Reply to specific comment
    (bind "r" 'reply-to-comment)))

;; ============================================
;; Conditional Keybindings
;; ============================================

;; Different bindings for different times of day
;; (just for fun/example)
(define (is-evening?)
  (>= (current-hour) 18))

(when (is-evening?)
  ;; Evening mode: darker theme
  (set-custom-theme
    '((foreground "#a89984")
      (background "#1d2021"))))
