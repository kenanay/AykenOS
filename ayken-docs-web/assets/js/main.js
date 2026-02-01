// AykenOS Documentation Website JavaScript

document.addEventListener('DOMContentLoaded', function() {
    // Mobile menu toggle
    const hamburger = document.querySelector('.hamburger');
    const navMenu = document.querySelector('.nav-menu');
    
    if (hamburger && navMenu) {
        hamburger.addEventListener('click', function() {
            hamburger.classList.toggle('active');
            navMenu.classList.toggle('active');
        });
    }
    
    // Close mobile menu when clicking on a link
    document.querySelectorAll('.nav-link').forEach(link => {
        link.addEventListener('click', () => {
            hamburger.classList.remove('active');
            navMenu.classList.remove('active');
        });
    });
    
    // Language toggle
    const langToggle = document.getElementById('lang-toggle');
    let currentLang = 'tr';
    
    if (langToggle) {
        langToggle.addEventListener('click', function() {
            currentLang = currentLang === 'tr' ? 'en' : 'tr';
            langToggle.textContent = currentLang === 'tr' ? 'EN' : 'TR';
            toggleLanguage(currentLang);
        });
    }
    
    // Smooth scrolling for anchor links
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                const headerOffset = 80;
                const elementPosition = target.getBoundingClientRect().top;
                const offsetPosition = elementPosition + window.pageYOffset - headerOffset;
                
                window.scrollTo({
                    top: offsetPosition,
                    behavior: 'smooth'
                });
            }
        });
    });
    
    // Active navigation highlighting
    const sections = document.querySelectorAll('section[id]');
    const navLinks = document.querySelectorAll('.nav-link[href^="#"]');
    
    function highlightNavigation() {
        let current = '';
        
        sections.forEach(section => {
            const sectionTop = section.getBoundingClientRect().top;
            const sectionHeight = section.offsetHeight;
            
            if (sectionTop <= 100 && sectionTop + sectionHeight > 100) {
                current = section.getAttribute('id');
            }
        });
        
        navLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('href') === `#${current}`) {
                link.classList.add('active');
            }
        });
    }
    
    window.addEventListener('scroll', highlightNavigation);
    
    // Fade in animation on scroll
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };
    
    const observer = new IntersectionObserver(function(entries) {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('fade-in-up');
            }
        });
    }, observerOptions);
    
    // Observe elements for animation
    document.querySelectorAll('.about-card, .arch-layer, .docs-category, .phase-status, .completed-phases, .metrics').forEach(el => {
        observer.observe(el);
    });
    
    // Progress bar animation
    function animateProgressBars() {
        const progressBars = document.querySelectorAll('.progress-fill');
        progressBars.forEach(bar => {
            const width = bar.style.width;
            bar.style.width = '0%';
            setTimeout(() => {
                bar.style.width = width;
            }, 500);
        });
    }
    
    // Animate progress bars when development section is visible
    const developmentSection = document.getElementById('development');
    if (developmentSection) {
        const developmentObserver = new IntersectionObserver(function(entries) {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    animateProgressBars();
                    developmentObserver.unobserve(entry.target);
                }
            });
        }, { threshold: 0.3 });
        
        developmentObserver.observe(developmentSection);
    }
    
    // Dynamic metrics counter animation
    function animateCounters() {
        const counters = document.querySelectorAll('.metric-number');
        counters.forEach(counter => {
            const target = parseInt(counter.textContent.replace(/\D/g, ''));
            const suffix = counter.textContent.replace(/\d/g, '');
            let current = 0;
            const increment = target / 50;
            
            const timer = setInterval(() => {
                current += increment;
                if (current >= target) {
                    counter.textContent = target + suffix;
                    clearInterval(timer);
                } else {
                    counter.textContent = Math.floor(current) + suffix;
                }
            }, 30);
        });
    }
    
    // Animate counters when metrics section is visible
    const metricsSection = document.querySelector('.metrics');
    if (metricsSection) {
        const metricsObserver = new IntersectionObserver(function(entries) {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    animateCounters();
                    metricsObserver.unobserve(entry.target);
                }
            });
        }, { threshold: 0.5 });
        
        metricsObserver.observe(metricsSection);
    }
    
    // Header background change on scroll
    const header = document.querySelector('.header');
    window.addEventListener('scroll', function() {
        if (window.scrollY > 100) {
            header.style.background = 'rgba(255, 255, 255, 0.95)';
            header.style.backdropFilter = 'blur(10px)';
        } else {
            header.style.background = 'var(--bg-color)';
            header.style.backdropFilter = 'none';
        }
    });
    
    // Copy to clipboard functionality for code blocks (if any)
    function addCopyButtons() {
        const codeBlocks = document.querySelectorAll('pre code');
        codeBlocks.forEach(block => {
            const button = document.createElement('button');
            button.className = 'copy-btn';
            button.textContent = 'Kopyala';
            button.addEventListener('click', () => {
                navigator.clipboard.writeText(block.textContent).then(() => {
                    button.textContent = 'Kopyalandı!';
                    setTimeout(() => {
                        button.textContent = 'Kopyala';
                    }, 2000);
                });
            });
            
            const wrapper = document.createElement('div');
            wrapper.className = 'code-wrapper';
            block.parentNode.insertBefore(wrapper, block);
            wrapper.appendChild(block);
            wrapper.appendChild(button);
        });
    }
    
    addCopyButtons();
    
    // Search functionality (basic)
    function initSearch() {
        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            searchInput.addEventListener('input', function(e) {
                const query = e.target.value.toLowerCase();
                const searchableElements = document.querySelectorAll('h1, h2, h3, p, li');
                
                searchableElements.forEach(element => {
                    const text = element.textContent.toLowerCase();
                    const parent = element.closest('section, .docs-category, .about-card');
                    
                    if (text.includes(query) || query === '') {
                        if (parent) parent.style.display = '';
                        element.style.display = '';
                    } else {
                        if (parent && !parent.querySelector(`*:not([style*="display: none"])`)) {
                            parent.style.display = 'none';
                        }
                    }
                });
            });
        }
    }
    
    initSearch();
    
    // Theme toggle (if implemented)
    function initThemeToggle() {
        const themeToggle = document.getElementById('theme-toggle');
        if (themeToggle) {
            const currentTheme = localStorage.getItem('theme') || 'light';
            document.documentElement.setAttribute('data-theme', currentTheme);
            
            themeToggle.addEventListener('click', function() {
                const theme = document.documentElement.getAttribute('data-theme');
                const newTheme = theme === 'light' ? 'dark' : 'light';
                
                document.documentElement.setAttribute('data-theme', newTheme);
                localStorage.setItem('theme', newTheme);
            });
        }
    }
    
    initThemeToggle();
});

// Language switching functionality
function toggleLanguage(lang) {
    const elements = document.querySelectorAll('[data-tr], [data-en]');
    
    elements.forEach(element => {
        if (lang === 'tr' && element.hasAttribute('data-tr')) {
            element.textContent = element.getAttribute('data-tr');
        } else if (lang === 'en' && element.hasAttribute('data-en')) {
            element.textContent = element.getAttribute('data-en');
        }
    });
    
    // Update document language
    document.documentElement.lang = lang;
    
    // Update page title
    if (lang === 'en') {
        document.title = 'AykenOS - Constitutional Operating System';
    } else {
        document.title = 'AykenOS - Anayasal İşletim Sistemi';
    }
}

// Utility functions
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

function throttle(func, limit) {
    let inThrottle;
    return function() {
        const args = arguments;
        const context = this;
        if (!inThrottle) {
            func.apply(context, args);
            inThrottle = true;
            setTimeout(() => inThrottle = false, limit);
        }
    }
}

// Performance optimization
const debouncedScroll = debounce(function() {
    // Scroll-based operations
}, 10);

window.addEventListener('scroll', debouncedScroll);

// Error handling
window.addEventListener('error', function(e) {
    console.error('JavaScript error:', e.error);
    // Could send to analytics or error reporting service
});

// Analytics placeholder (replace with actual analytics code)
function trackEvent(category, action, label) {
    // Google Analytics, Matomo, or other analytics
    console.log('Event tracked:', { category, action, label });
}

// Track navigation clicks
document.querySelectorAll('.nav-link').forEach(link => {
    link.addEventListener('click', function() {
        trackEvent('Navigation', 'Click', this.textContent);
    });
});

// Track external links
document.querySelectorAll('a[target="_blank"]').forEach(link => {
    link.addEventListener('click', function() {
        trackEvent('External Link', 'Click', this.href);
    });
});