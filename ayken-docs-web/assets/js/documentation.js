// AykenOS Documentation JavaScript

class DocumentationApp {
    constructor() {
        this.currentSection = '1-1';
        this.theme = localStorage.getItem('theme') || 'light';
        this.glossaryTerms = new Map();
        
        this.init();
    }
    
    init() {
        this.setupTheme();
        this.setupNavigation();
        this.setupMobileSidebar();
        this.setupSearch();
        this.setupGlossary();
        this.setupCodeCopy();
        this.setupTermTooltips();
        this.setupKeyboardShortcuts();
        
        // Load initial section
        this.showSection(this.currentSection);
        this.updateActiveNav(this.currentSection);
    }
    
    setupTheme() {
        const themeBtn = document.getElementById('theme-toggle');
        const body = document.body;
        
        // Apply saved theme
        body.setAttribute('data-theme', this.theme);
        this.updateThemeButton();
        
        themeBtn.addEventListener('click', () => {
            this.theme = this.theme === 'light' ? 'dark' : 'light';
            body.setAttribute('data-theme', this.theme);
            localStorage.setItem('theme', this.theme);
            this.updateThemeButton();
        });
    }
    
    updateThemeButton() {
        const themeBtn = document.getElementById('theme-toggle');
        themeBtn.textContent = this.theme === 'light' ? '🌙' : '☀️';
        themeBtn.title = this.theme === 'light' ? 'Karanlık Tema' : 'Aydınlık Tema';
    }
    
    setupMobileSidebar() {
        const sidebarToggle = document.getElementById('sidebar-toggle');
        const sidebar = document.querySelector('.doc-sidebar');
        
        sidebarToggle.addEventListener('click', () => {
            sidebar.classList.toggle('active');
        });
        
        // Close sidebar when clicking outside
        document.addEventListener('click', (e) => {
            if (window.innerWidth <= 768) {
                if (!sidebar.contains(e.target) && !sidebarToggle.contains(e.target)) {
                    sidebar.classList.remove('active');
                }
            }
        });
        
        // Close sidebar when navigating to a section on mobile
        const navLinks = document.querySelectorAll('.doc-nav a');
        navLinks.forEach(link => {
            link.addEventListener('click', () => {
                if (window.innerWidth <= 768) {
                    sidebar.classList.remove('active');
                }
            });
        });
    }
    
    setupNavigation() {
        // Sidebar navigation
        const navLinks = document.querySelectorAll('.doc-nav a');
        navLinks.forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const section = link.getAttribute('data-section');
                this.navigateToSection(section);
            });
        });
        
        // Section navigation buttons
        document.addEventListener('click', (e) => {
            if (e.target.classList.contains('next-btn')) {
                const nextSection = e.target.getAttribute('data-next');
                if (nextSection) {
                    this.navigateToSection(nextSection);
                }
            } else if (e.target.classList.contains('prev-btn')) {
                const prevSection = e.target.getAttribute('data-prev');
                if (prevSection) {
                    this.navigateToSection(prevSection);
                }
            }
        });
        
        // Handle browser back/forward
        window.addEventListener('popstate', (e) => {
            if (e.state && e.state.section) {
                this.showSection(e.state.section);
                this.updateActiveNav(e.state.section);
            }
        });
    }
    
    navigateToSection(section) {
        this.currentSection = section;
        this.showSection(section);
        this.updateActiveNav(section);
        
        // Update URL without page reload
        const url = new URL(window.location);
        url.hash = section;
        history.pushState({ section }, '', url);
        
        // Scroll to top
        window.scrollTo({ top: 0, behavior: 'smooth' });
    }
    
    showSection(sectionId) {
        // Hide all sections
        const sections = document.querySelectorAll('.doc-section');
        sections.forEach(section => {
            section.classList.remove('active');
        });
        
        // Show target section
        const targetSection = document.getElementById(sectionId);
        if (targetSection) {
            targetSection.classList.add('active');
            
            // Update page title
            const sectionTitle = targetSection.querySelector('h1').textContent;
            document.title = `${sectionTitle} - AykenOS Dokümantasyon`;
        }
    }
    
    updateActiveNav(sectionId) {
        // Remove active class from all nav links
        const navLinks = document.querySelectorAll('.doc-nav a');
        navLinks.forEach(link => {
            link.classList.remove('active');
        });
        
        // Add active class to current section
        const activeLink = document.querySelector(`[data-section="${sectionId}"]`);
        if (activeLink) {
            activeLink.classList.add('active');
        }
    }
    
    setupSearch() {
        const searchInput = document.getElementById('doc-search');
        const navSections = document.querySelectorAll('.nav-section');
        
        searchInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase().trim();
            
            if (query === '') {
                // Show all sections
                navSections.forEach(section => {
                    section.style.display = 'block';
                });
                return;
            }
            
            navSections.forEach(section => {
                const sectionText = section.textContent.toLowerCase();
                const hasMatch = sectionText.includes(query);
                section.style.display = hasMatch ? 'block' : 'none';
            });
        });
    }
    
    setupGlossary() {
        const glossaryBtn = document.getElementById('glossary-btn');
        const glossaryModal = document.getElementById('glossary-modal');
        const closeBtn = glossaryModal.querySelector('.close-btn');
        const glossarySearch = document.getElementById('glossary-search');
        const glossaryItems = document.querySelectorAll('.glossary-item');
        
        // Load glossary terms
        this.loadGlossaryTerms();
        
        // Open modal
        glossaryBtn.addEventListener('click', () => {
            glossaryModal.classList.add('active');
            document.body.style.overflow = 'hidden';
            glossarySearch.focus();
        });
        
        // Close modal
        const closeModal = () => {
            glossaryModal.classList.remove('active');
            document.body.style.overflow = '';
            glossarySearch.value = '';
            this.filterGlossary('');
        };
        
        closeBtn.addEventListener('click', closeModal);
        
        glossaryModal.addEventListener('click', (e) => {
            if (e.target === glossaryModal) {
                closeModal();
            }
        });
        
        // Search glossary
        glossarySearch.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase().trim();
            this.filterGlossary(query);
        });
        
        // ESC key to close
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && glossaryModal.classList.contains('active')) {
                closeModal();
            }
        });
    }
    
    loadGlossaryTerms() {
        const glossaryItems = document.querySelectorAll('.glossary-item');
        glossaryItems.forEach(item => {
            const term = item.getAttribute('data-term');
            const title = item.querySelector('h4').textContent;
            const description = item.querySelector('p').textContent;
            
            this.glossaryTerms.set(term, {
                title,
                description,
                element: item
            });
        });
    }
    
    filterGlossary(query) {
        const glossaryItems = document.querySelectorAll('.glossary-item');
        
        if (query === '') {
            glossaryItems.forEach(item => {
                item.style.display = 'block';
            });
            return;
        }
        
        glossaryItems.forEach(item => {
            const title = item.querySelector('h4').textContent.toLowerCase();
            const description = item.querySelector('p').textContent.toLowerCase();
            const hasMatch = title.includes(query) || description.includes(query);
            
            item.style.display = hasMatch ? 'block' : 'none';
        });
    }
    
    setupCodeCopy() {
        const copyButtons = document.querySelectorAll('.copy-btn');
        
        copyButtons.forEach(btn => {
            btn.addEventListener('click', async () => {
                const codeId = btn.getAttribute('data-copy');
                const codeElement = document.getElementById(codeId);
                
                if (codeElement) {
                    try {
                        await navigator.clipboard.writeText(codeElement.textContent);
                        
                        // Visual feedback
                        const originalText = btn.textContent;
                        btn.textContent = '✅';
                        btn.style.background = 'rgba(39, 174, 96, 0.8)';
                        
                        setTimeout(() => {
                            btn.textContent = originalText;
                            btn.style.background = '';
                        }, 2000);
                        
                    } catch (err) {
                        console.error('Copy failed:', err);
                        
                        // Fallback for older browsers
                        const textArea = document.createElement('textarea');
                        textArea.value = codeElement.textContent;
                        document.body.appendChild(textArea);
                        textArea.select();
                        document.execCommand('copy');
                        document.body.removeChild(textArea);
                        
                        btn.textContent = '✅';
                        setTimeout(() => {
                            btn.textContent = '📋';
                        }, 2000);
                    }
                }
            });
        });
    }
    
    setupTermTooltips() {
        const terms = document.querySelectorAll('term');
        
        terms.forEach(term => {
            const termKey = term.getAttribute('data-term');
            const glossaryData = this.glossaryTerms.get(termKey);
            
            if (glossaryData) {
                // Create tooltip
                const tooltip = document.createElement('div');
                tooltip.className = 'term-tooltip';
                tooltip.innerHTML = `
                    <strong>${glossaryData.title}</strong>
                    <p>${glossaryData.description}</p>
                `;
                
                // Add tooltip styles
                tooltip.style.cssText = `
                    position: absolute;
                    background: var(--bg-color);
                    border: 2px solid var(--secondary-color);
                    border-radius: 8px;
                    padding: 12px;
                    max-width: 300px;
                    font-size: 14px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                    z-index: 1000;
                    display: none;
                    pointer-events: none;
                `;
                
                document.body.appendChild(tooltip);
                
                // Show/hide tooltip
                term.addEventListener('mouseenter', (e) => {
                    const rect = term.getBoundingClientRect();
                    tooltip.style.display = 'block';
                    tooltip.style.left = rect.left + 'px';
                    tooltip.style.top = (rect.bottom + 8) + 'px';
                });
                
                term.addEventListener('mouseleave', () => {
                    tooltip.style.display = 'none';
                });
                
                // Click to open glossary
                term.addEventListener('click', () => {
                    document.getElementById('glossary-btn').click();
                    setTimeout(() => {
                        const searchInput = document.getElementById('glossary-search');
                        searchInput.value = glossaryData.title;
                        this.filterGlossary(glossaryData.title.toLowerCase());
                    }, 100);
                });
            }
        });
    }
    
    setupKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Only handle shortcuts when not in input fields
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
                return;
            }
            
            switch (e.key) {
                case 'ArrowLeft':
                    if (e.altKey) {
                        e.preventDefault();
                        this.navigateToPrevious();
                    }
                    break;
                    
                case 'ArrowRight':
                    if (e.altKey) {
                        e.preventDefault();
                        this.navigateToNext();
                    }
                    break;
                    
                case '/':
                    e.preventDefault();
                    document.getElementById('doc-search').focus();
                    break;
                    
                case 'g':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        document.getElementById('glossary-btn').click();
                    }
                    break;
                    
                case 't':
                    if (e.ctrlKey || e.metaKey) {
                        e.preventDefault();
                        document.getElementById('theme-toggle').click();
                    }
                    break;
            }
        });
    }
    
    navigateToPrevious() {
        const currentBtn = document.querySelector('.doc-section.active .prev-btn');
        if (currentBtn) {
            const prevSection = currentBtn.getAttribute('data-prev');
            if (prevSection) {
                this.navigateToSection(prevSection);
            }
        }
    }
    
    navigateToNext() {
        const currentBtn = document.querySelector('.doc-section.active .next-btn');
        if (currentBtn) {
            const nextSection = currentBtn.getAttribute('data-next');
            if (nextSection) {
                this.navigateToSection(nextSection);
            }
        }
    }
}

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new DocumentationApp();
});

// Handle initial hash navigation
window.addEventListener('load', () => {
    const hash = window.location.hash.substring(1);
    if (hash) {
        const app = window.documentationApp;
        if (app) {
            app.navigateToSection(hash);
        }
    }
});

// Smooth scrolling for anchor links
document.addEventListener('click', (e) => {
    if (e.target.tagName === 'A' && e.target.getAttribute('href').startsWith('#')) {
        e.preventDefault();
        const targetId = e.target.getAttribute('href').substring(1);
        const targetElement = document.getElementById(targetId);
        
        if (targetElement) {
            targetElement.scrollIntoView({
                behavior: 'smooth',
                block: 'start'
            });
        }
    }
});

// Add loading animation
window.addEventListener('beforeunload', () => {
    document.body.style.opacity = '0.7';
});

// Performance optimization: Lazy load sections
const observerOptions = {
    root: null,
    rootMargin: '50px',
    threshold: 0.1
};

const sectionObserver = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            entry.target.classList.add('visible');
        }
    });
}, observerOptions);

// Observe all sections for lazy loading
document.addEventListener('DOMContentLoaded', () => {
    const sections = document.querySelectorAll('.doc-section');
    sections.forEach(section => {
        sectionObserver.observe(section);
    });
});

// Add print functionality
function printDocumentation() {
    // Show all sections for printing
    const sections = document.querySelectorAll('.doc-section');
    sections.forEach(section => {
        section.style.display = 'block';
    });
    
    window.print();
    
    // Restore original display after printing
    setTimeout(() => {
        sections.forEach(section => {
            if (!section.classList.contains('active')) {
                section.style.display = 'none';
            }
        });
    }, 1000);
}

// Export for global access
window.printDocumentation = printDocumentation;