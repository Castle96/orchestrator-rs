#!/bin/bash

# TLS Certificate Setup Script for ARM Hypervisor
# Supports both self-signed and Let's Encrypt certificates

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Configuration
SSL_DIR="/etc/arm-hypervisor/ssl"
CONFIG_FILE="/etc/arm-hypervisor/config.toml"

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root"
        exit 1
    fi
}

setup_ssl_directory() {
    log_info "Setting up SSL directory..."
    mkdir -p "$SSL_DIR"
    chmod 700 "$SSL_DIR"
    log_success "SSL directory created: $SSL_DIR"
}

generate_self_signed() {
    log_info "Generating self-signed certificate..."
    
    # Get hostname
    HOSTNAME=$(hostname -f)
    if [[ -z "$HOSTNAME" ]]; then
        HOSTNAME=$(hostname)
    fi
    if [[ -z "$HOSTNAME" ]]; then
        HOSTNAME="localhost"
    fi
    
    # Generate certificate
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout "$SSL_DIR/server.key" \
        -out "$SSL_DIR/server.crt" \
        -subj "/C=US/ST=State/L=City/O=ARM Hypervisor/CN=$HOSTNAME" \
        2>/dev/null
    
    # Set permissions
    chmod 600 "$SSL_DIR/server.key"
    chmod 644 "$SSL_DIR/server.crt"
    chown root:root "$SSL_DIR"/*
    
    log_success "Self-signed certificate generated"
    log_info "Certificate details:"
    openssl x509 -in "$SSL_DIR/server.crt" -text -noout | grep -A 2 "Subject:"
}

setup_letsencrypt() {
    DOMAIN="$1"
    
    if [[ -z "$DOMAIN" ]]; then
        log_error "Domain name required for Let's Encrypt"
        return 1
    fi
    
    log_info "Setting up Let's Encrypt certificate for: $DOMAIN"
    
    # Install certbot if not present
    if ! command -v certbot &> /dev/null; then
        log_info "Installing certbot..."
        apt update
        apt install -y certbot
    fi
    
    # Stop arm-hypervisor service if running
    if systemctl is-active --quiet arm-hypervisor; then
        log_info "Stopping arm-hypervisor service..."
        systemctl stop arm-hypervisor
    fi
    
    # Obtain certificate
    certbot certonly --standalone -d "$DOMAIN" --non-interactive --agree-tos --email "admin@$DOMAIN"
    
    # Copy certificates to SSL directory
    cp "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" "$SSL_DIR/server.crt"
    cp "/etc/letsencrypt/live/$DOMAIN/privkey.pem" "$SSL_DIR/server.key"
    
    # Set permissions
    chmod 600 "$SSL_DIR/server.key"
    chmod 644 "$SSL_DIR/server.crt"
    chown root:root "$SSL_DIR"/*
    
    log_success "Let's Encrypt certificate obtained"
    log_info "Auto-renewal is configured by certbot"
    
    # Restart service if it was running
    if systemctl is-enabled --quiet arm-hypervisor; then
        log_info "Restarting arm-hypervisor service..."
        systemctl start arm-hypervisor
    fi
}

update_config() {
    log_info "Updating configuration file..."
    
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log_error "Configuration file not found: $CONFIG_FILE"
        return 1
    fi
    
    # Check if TLS section exists
    if grep -q "^\[server.tls\]" "$CONFIG_FILE"; then
        log_info "TLS section already exists in config"
    else
        # Add TLS section to config
        cat >> "$CONFIG_FILE" <<'EOF'

[server.tls]
cert_file = "/etc/arm-hypervisor/ssl/server.crt"
key_file = "/etc/arm-hypervisor/ssl/server.key"
EOF
    fi
    
    # Update paths if they're different
    sed -i "s|cert_file = .*|cert_file = \"$SSL_DIR/server.crt\"|g" "$CONFIG_FILE"
    sed -i "s|key_file = .*|key_file = \"$SSL_DIR/server.key\"|g" "$CONFIG_FILE"
    
    log_success "Configuration updated"
}

verify_certificate() {
    log_info "Verifying certificate..."
    
    if [[ ! -f "$SSL_DIR/server.crt" || ! -f "$SSL_DIR/server.key" ]]; then
        log_error "Certificate files not found"
        return 1
    fi
    
    # Check certificate validity
    if openssl x509 -in "$SSL_DIR/server.crt" -noout -checkend 86400; then
        log_success "Certificate is valid"
    else
        log_warning "Certificate expires within 24 hours"
    fi
    
    # Check key match
    CERT_MD5=$(openssl x509 -noout -modulus -in "$SSL_DIR/server.crt" | openssl md5)
    KEY_MD5=$(openssl rsa -noout -modulus -in "$SSL_DIR/server.key" | openssl md5 2>/dev/null)
    
    if [[ "$CERT_MD5" == "$KEY_MD5" ]]; then
        log_success "Certificate and key match"
    else
        log_error "Certificate and key do not match"
        return 1
    fi
}

show_certificate_info() {
    log_info "Certificate Information:"
    echo "=========================="
    
    if [[ -f "$SSL_DIR/server.crt" ]]; then
        echo "Subject:"
        openssl x509 -in "$SSL_DIR/server.crt" -noout -subject | sed 's/subject=//'
        echo
        echo "Issuer:"
        openssl x509 -in "$SSL_DIR/server.crt" -noout -issuer | sed 's/issuer=//'
        echo
        echo "Valid From:"
        openssl x509 -in "$SSL_DIR/server.crt" -noout -startdate | sed 's/notBefore=//'
        echo
        echo "Valid Until:"
        openssl x509 -in "$SSL_DIR/server.crt" -noout -enddate | sed 's/notAfter=//'
        echo
        echo "Fingerprint:"
        openssl x509 -in "$SSL_DIR/server.crt" -noout -fingerprint
    else
        log_error "Certificate file not found"
    fi
}

setup_renewal() {
    log_info "Setting up certificate renewal..."
    
    if [[ -d "/etc/letsencrypt" ]]; then
        log_info "Let's Encrypt detected, renewal is automatic"
        
        # Create renewal hook to restart service
        cat > /etc/letsencrypt/renewal-hooks/deploy/arm-hypervisor.sh <<'EOF'
#!/bin/bash
DOMAIN=$(echo $RENEWED_DOMAINS | cut -d' ' -f1)
if [[ -n "$DOMAIN" ]]; then
    cp "/etc/letsencrypt/live/$DOMAIN/fullchain.pem" "/etc/arm-hypervisor/ssl/server.crt"
    cp "/etc/letsencrypt/live/$DOMAIN/privkey.pem" "/etc/arm-hypervisor/ssl/server.key"
    systemctl restart arm-hypervisor
fi
EOF
        
        chmod +x /etc/letsencrypt/renewal-hooks/deploy/arm-hypervisor.sh
        log_success "Renewal hook configured"
    else
        log_warning "Self-signed certificates cannot be auto-renewed"
        log_info "Run this script again before expiration"
    fi
}

# Main execution
main() {
    log_info "ARM Hypervisor TLS Setup"
    echo "=========================="
    
    check_root
    setup_ssl_directory
    
    echo
    echo "Choose certificate type:"
    echo "1) Self-signed certificate (for testing/internal use)"
    echo "2) Let's Encrypt certificate (for production with domain)"
    echo "3) Verify existing certificate"
    echo "4) Show certificate info"
    echo
    
    read -p "Enter choice (1-4): " -n 1 -r
    echo
    echo
    
    case $REPLY in
        1)
            generate_self_signed
            update_config
            verify_certificate
            log_success "Self-signed certificate setup complete"
            ;;
        2)
            read -p "Enter domain name: " DOMAIN
            setup_letsencrypt "$DOMAIN"
            update_config
            verify_certificate
            setup_renewal
            log_success "Let's Encrypt certificate setup complete"
            ;;
        3)
            verify_certificate
            ;;
        4)
            show_certificate_info
            ;;
        *)
            log_error "Invalid choice"
            exit 1
            ;;
    esac
    
    echo
    log_info "TLS setup completed!"
    log_info "Restart arm-hypervisor service to apply changes:"
    echo "  sudo systemctl restart arm-hypervisor"
}

# Run main function
main "$@"