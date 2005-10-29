<?xml version="1.0" encoding="ISO-8859-1"?>
<xsl:stylesheet version="1.0"
		xmlns:check="http://check.sourceforge.net/ns"
		xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
		xmlns="http://www.w3.org/1999/xhtml">

<xsl:param name="hostsystem"/>
		
<xsl:output method="xml" version="1.0" encoding="utf-8"
	doctype-public="html PUBLIC -//W3C//DTD XHTML 1.1//EN"
	doctype-system="http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd"/>

<xsl:template match="check:testsuites">
	<html>
		<head>
			<title>Unit Test Results</title>
			<meta http-equiv="content-type" content="text/html; charset=utf-8"/>
			<link type="text/css" rel="stylesheet" href="lf.css"/>
			<link type="text/css" rel="stylesheet" href="test.css"/>
			<script type="text/javascript" src="showhide.js">&#160;</script>
		</head>
		<body>
			<h2 class="title"><xsl:value-of select="$hostsystem"/></h2>
			<xsl:apply-templates select="check:datetime"/>	
			<xsl:apply-templates select="check:suite"/>	
		</body>
	</html>
</xsl:template>

<xsl:template match="check:datetime">
	<h3 class="title">Date: <xsl:value-of select="."/></h3>
</xsl:template>

<xsl:template match="check:title">
	<a name="{generate-id(.)}">
		<h3 class="suite">
			Suite: <xsl:value-of select="."/>
			[ <a class="disclosure" href="#{generate-id(.)}"
				onclick="
					show_div('{generate-id(./..)}-full');
					hide_div('{generate-id(./..)}-sum');">+</a>
			/
			<a class="disclosure" href="#{generate-id(.)}"
				onclick="
					hide_div('{generate-id(./..)}-full');
					show_div('{generate-id(./..)}-sum');">-</a> ]
		</h3>
	</a>
</xsl:template>

<xsl:template match="check:suite">
	<xsl:apply-templates select="check:title"/>

	<div id="{generate-id(.)}-sum" class="results">
		<p><span class="failed"><xsl:value-of select="count(check:test[@result='failure'])"/> Failed</span> / <span class="success"><xsl:value-of select="count(check:test[@result='success'])"/> Passed</span></p>
	</div>
	<div id="{generate-id(.)}-full" class="results">
		<h4 class="failed">Failed Tests: <xsl:value-of select="count(check:test[@result='failure'])"/></h4>
		<xsl:apply-templates select="check:test[@result='failure']"/>

		<h4 class="success">Passed Tests: <xsl:value-of select="count(check:test[@result='success'])"/></h4>
		<xsl:apply-templates select="check:test[@result='success']"/>
	</div>
	<script type="text/javascript">
		hide_div('<xsl:value-of select="generate-id(.)"/>-full');
		show_div('<xsl:value-of select="generate-id(.)"/>-sum');
	</script>
</xsl:template>

<xsl:template match="check:test">
	<div class="test">
		<xsl:apply-templates select="check:id"/>
		<xsl:apply-templates select="check:description"/>
		<xsl:if test="@result = 'failure'">
			<xsl:apply-templates select="check:message"/>
			<xsl:apply-templates select="check:fn"/>
		</xsl:if>
	</div>
</xsl:template>

<xsl:template match="check:id">
	<p class="testname"><xsl:apply-templates/></p>
</xsl:template>

<xsl:template match="check:description">
	<p class="test">Description: <xsl:apply-templates/></p>
</xsl:template>

<xsl:template match="check:message">
	<p class="test">Message: <xsl:apply-templates/></p>
</xsl:template>

<xsl:template match="check:fn">
	<p class="test">File/Line: <xsl:apply-templates/></p>
</xsl:template>

	
</xsl:stylesheet>
