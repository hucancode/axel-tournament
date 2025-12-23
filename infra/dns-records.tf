# DNS Records for Application
# These will be created after the ALB is provisioned

# Note: The actual DNS records pointing to the ALB will be created
# by the Makefile after the ingress creates the ALB, because we need
# the ALB hostname which is only known after deployment.

# This file serves as a placeholder and documentation.
# The Makefile will use `kubectl` to get the ALB hostname and then
# use Terraform or AWS CLI to create the DNS records dynamically.
